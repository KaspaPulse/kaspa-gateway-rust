//! Construct and persist explicit test-result artifacts. No desktop/runtime launch.
use serde_json::Value;
use std::io::Read;
use std::path::PathBuf;
use xtask::zero_touch_evidence::{EvidenceResult, read_json, result};
use xtask::zero_touch_result_io::{artifact_target, atomic_write};

const USAGE: &str = "kgw-zero-touch-result <build|failure|write|build-write> --request <JSON_PATH|-> [--artifact-directory PATH] [--file-name zero-touch-result.json]";
fn run() -> EvidenceResult<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or_else(|| USAGE.to_owned())?;
    if command == "--help" || command == "-h" {
        println!("{USAGE}");
        return Ok(());
    }
    let mut request = None;
    let mut directory = None;
    let mut filename = None;
    while let Some(option) = args.next() {
        let destination = match option.as_str() {
            "--request" => &mut request,
            "--artifact-directory" => &mut directory,
            "--file-name" => &mut filename,
            _ => return Err(format!("Unknown argument: {option}")),
        };
        if destination.is_some() {
            return Err(format!("Duplicate argument: {option}"));
        }
        *destination = Some(
            args.next()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("Missing value for {option}"))?,
        );
    }
    let request = request.ok_or_else(|| "--request is required".to_owned())?;
    let value = if request == "-" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .take(32 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() > 32 * 1024 * 1024 {
            return Err("JSON request exceeds32MiB".to_owned());
        }
        let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
        serde_json::from_slice(bytes).map_err(|error| format!("Invalid JSON request: {error}"))?
    } else {
        let path = PathBuf::from(request);
        if !path.is_file() {
            return Err("Request JSON file is missing".to_owned());
        }
        read_json(&path)?
    };
    let output: Value = match command.as_str() {
        "build" => result::build(&value)?,
        "failure" => result::failure(&value)?,
        "write" => {
            let directory = PathBuf::from(
                directory.ok_or_else(|| "--artifact-directory is required for write".to_owned())?,
            );
            let path = artifact_target(
                &directory,
                filename.as_deref().unwrap_or("zero-touch-result.json"),
            )?;
            atomic_write(&path, &value)?
        }
        "build-write" => {
            if directory.is_some() || filename.is_some() {
                return Err(
                    "build-write derives its single target from artifact_directory in the request"
                        .to_owned(),
                );
            }
            let built = result::build(&value)?;
            let directory = built["artifact_directory"]
                .as_str()
                .ok_or_else(|| "Invalid artifact directory".to_owned())?;
            atomic_write(
                &artifact_target(std::path::Path::new(directory), "zero-touch-result.json")?,
                &built,
            )?;
            built
        }
        _ => return Err(format!("Unknown command: {command}\n{USAGE}")),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
