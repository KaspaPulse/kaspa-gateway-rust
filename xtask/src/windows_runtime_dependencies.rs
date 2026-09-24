use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedImports {
    imported: Vec<String>,
    external_msvc_runtime: Vec<String>,
}

#[derive(Debug)]
struct Options {
    executable: PathBuf,
    dumpbin_path: Option<PathBuf>,
    report_path: Option<PathBuf>,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let options = parse_options(args)?;
    run(&options)
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<Options, String> {
    let mut executable = None;
    let mut dumpbin_path = None;
    let mut report_path = None;

    while let Some(arg) = args.next() {
        let value = match arg.as_str() {
            "--executable" | "--dumpbin-path" | "--report-path" => args
                .next()
                .ok_or_else(|| format!("{arg} requires a value"))?,
            _ => {
                return Err(format!(
                    "unknown verify-windows-runtime-dependencies argument: {arg}"
                ));
            }
        };
        match arg.as_str() {
            "--executable" => executable = Some(PathBuf::from(value)),
            "--dumpbin-path" => dumpbin_path = Some(PathBuf::from(value)),
            "--report-path" => report_path = Some(PathBuf::from(value)),
            _ => unreachable!(),
        }
    }

    Ok(Options {
        executable: executable.ok_or_else(|| "--executable is required".to_owned())?,
        dumpbin_path,
        report_path,
    })
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let _ = options;
        return Err("verify-windows-runtime-dependencies requires Windows".to_owned());
    }

    #[cfg(windows)]
    {
        run_windows(options)
    }
}
#[cfg(windows)]
fn run_windows(options: &Options) -> Result<String, String> {
    let executable = fs::canonicalize(&options.executable).map_err(|error| {
        format!(
            "Cannot resolve executable {}: {error}",
            options.executable.display()
        )
    })?;
    if !executable.is_file() {
        return Err(format!(
            "Executable is not a file: {}",
            executable.display()
        ));
    }

    let dumpbin = match &options.dumpbin_path {
        Some(path) => {
            let resolved = fs::canonicalize(path)
                .map_err(|error| format!("Cannot resolve dumpbin {}: {error}", path.display()))?;
            if !resolved.is_file() {
                return Err(format!(
                    "Cannot inspect PE imports without dumpbin.exe: {}",
                    resolved.display()
                ));
            }
            resolved
        }
        None => discover_dumpbin()?,
    };

    let output = Command::new(&dumpbin)
        .arg("/DEPENDENTS")
        .arg(&executable)
        .output()
        .map_err(|error| format!("PE dependency inspection failed to start: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() {
        stdout.into_owned()
    } else if stdout.is_empty() {
        stderr.into_owned()
    } else {
        format!("{stdout}\n{stderr}")
    };
    if !output.status.success() {
        return Err(format!(
            "PE dependency inspection failed: {}",
            combined.trim()
        ));
    }

    let parsed = parse_dumpbin_dependents(&combined)?;
    let report = json!({
        "executable": display_windows_path(&executable),
        "sha256": sha256_file(&executable)?,
        "checkedAtUtc": OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|error| format!("checkedAtUtc formatting failed: {error}"))?,
        "importedDlls": parsed.imported,
        "externalMsvcRuntimeDlls": parsed.external_msvc_runtime,
        "passed": parsed.external_msvc_runtime.is_empty(),
    });

    let mut rendered = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("runtime dependency report serialization failed: {error}"))?;
    rendered.push('\n');

    if let Some(report_path) = &options.report_path {
        if let Some(parent) = report_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "Cannot create report directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(report_path, &rendered).map_err(|error| {
            format!(
                "Cannot write runtime dependency report {}: {error}",
                report_path.display()
            )
        })?;
    }

    if !report["externalMsvcRuntimeDlls"]
        .as_array()
        .is_some_and(|items| items.is_empty())
    {
        let names = report["externalMsvcRuntimeDlls"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Windows artifact requires separately installed MSVC runtime DLLs: {names}"
        ));
    }

    Ok(rendered.trim_end().to_owned())
}
#[cfg(windows)]
fn discover_dumpbin() -> Result<PathBuf, String> {
    if let Some(found) = find_command("dumpbin.exe") {
        return Ok(found);
    }

    let mut roots = Vec::new();
    if let Some(value) = env::var_os("ProgramFiles(x86)") {
        roots.push(PathBuf::from(value));
    }
    if let Some(drive) = env::var_os("SystemDrive") {
        roots.push(PathBuf::from(drive).join("Program Files (x86)"));
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)"));
    if let Some(value) = env::var_os("ProgramFiles") {
        roots.push(PathBuf::from(value));
    }

    let vswhere = roots
        .into_iter()
        .map(|root| {
            root.join("Microsoft Visual Studio")
                .join("Installer")
                .join("vswhere.exe")
        })
        .find(|path| path.is_file())
        .ok_or_else(|| "dumpbin.exe and vswhere.exe are unavailable.".to_owned())?;

    let output = Command::new(&vswhere)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-find",
            r"VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe",
        ])
        .output()
        .map_err(|error| format!("vswhere.exe failed to start: {error}"))?;
    if !output.status.success() {
        return Err("Cannot inspect PE imports without dumpbin.exe.".to_owned());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let candidate = PathBuf::from(line);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("Cannot inspect PE imports without dumpbin.exe.".to_owned())
}

#[cfg(windows)]
fn find_command(name: &str) -> Option<PathBuf> {
    let output = Command::new("where.exe").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn parse_dumpbin_dependents(text: &str) -> Result<ParsedImports, String> {
    let mut imports = BTreeSet::new();
    for line in text.lines() {
        if line.is_empty() || !line.starts_with(char::is_whitespace) {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.split_whitespace().count() != 1
            || !trimmed.to_ascii_lowercase().ends_with(".dll")
        {
            continue;
        }
        imports.insert(trimmed.to_ascii_uppercase());
    }
    if imports.is_empty() {
        return Err("No PE imports were parsed; refusing to report a pass.".to_owned());
    }

    let imported: Vec<_> = imports.into_iter().collect();
    let external_msvc_runtime = imported
        .iter()
        .filter(|name| is_external_msvc_runtime(name))
        .cloned()
        .collect();

    Ok(ParsedImports {
        imported,
        external_msvc_runtime,
    })
}

fn is_external_msvc_runtime(name: &str) -> bool {
    if !name.ends_with(".DLL") {
        return false;
    }
    for prefix in ["MSVCP", "MSVCR", "VCRUNTIME", "CONCRT"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            return rest.as_bytes().first().is_some_and(u8::is_ascii_digit);
        }
    }
    false
}
fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("Cannot open {} for SHA-256: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Cannot read {} for SHA-256: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(windows)]
fn display_windows_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unique_dumpbin_imports_case_insensitively() {
        let text = r#"
Microsoft (R) COFF/PE Dumper Version

  Image has the following dependencies:

    KERNEL32.dll
    bcryptprimitives.DLL
    KERNEL32.dll
    API-MS-WIN-CORE-SYNCH-L1-2-0.dll
"#;
        let parsed = parse_dumpbin_dependents(text).unwrap();
        assert_eq!(
            parsed.imported,
            vec![
                "API-MS-WIN-CORE-SYNCH-L1-2-0.DLL",
                "BCRYPTPRIMITIVES.DLL",
                "KERNEL32.DLL",
            ]
        );
        assert!(parsed.external_msvc_runtime.is_empty());
    }

    #[test]
    fn detects_external_msvc_runtime_families() {
        for name in [
            "MSVCP140.DLL",
            "MSVCR120.DLL",
            "VCRUNTIME140_1.DLL",
            "CONCRT140.DLL",
        ] {
            assert!(is_external_msvc_runtime(name), "{name}");
        }
        for name in ["KERNEL32.DLL", "UCRTBASE.DLL", "VCRUNTIME.DLL", "MSVCP.DLL"] {
            assert!(!is_external_msvc_runtime(name), "{name}");
        }
    }

    #[test]
    fn parser_fails_closed_when_no_imports_are_found() {
        let error =
            parse_dumpbin_dependents("Image has the following dependencies:\n").unwrap_err();
        assert!(error.contains("No PE imports were parsed"));
    }

    #[test]
    fn runtime_detection_is_reflected_in_parsed_result() {
        let text = "    KERNEL32.dll\n    VCRUNTIME140.dll\n";
        let parsed = parse_dumpbin_dependents(text).unwrap();
        assert_eq!(
            parsed.external_msvc_runtime,
            vec!["VCRUNTIME140.DLL".to_owned()]
        );
    }
}
