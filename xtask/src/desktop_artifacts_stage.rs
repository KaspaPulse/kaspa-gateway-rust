#![cfg_attr(
    target_os = "linux",
    allow(dead_code, unused_imports, unused_variables)
)]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Stdio;
use std::process::{Command, ExitStatus, Output};
use std::thread;
use std::time::Duration;

const PROTOC_VERSION: &str = "35.1";
#[cfg(windows)]
const WINDOWS_PROTOC_SHA256: &str =
    "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082";
#[cfg(target_os = "macos")]
const MACOS_PROTOC_SHA256: &str =
    "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b";
const EXPECTED_NPM: &str = "11.17.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Windows,
    Macos,
    PreserveWindowsSbom,
    PreserveWindowsProvenance,
    PreserveMacosSbom,
    PreserveMacosProvenance,
}
impl Stage {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "windows" => Ok(Self::Windows),
            "macos" => Ok(Self::Macos),
            "preserve-windows-sbom" => Ok(Self::PreserveWindowsSbom),
            "preserve-windows-provenance" => Ok(Self::PreserveWindowsProvenance),
            "preserve-macos-sbom" => Ok(Self::PreserveMacosSbom),
            "preserve-macos-provenance" => Ok(Self::PreserveMacosProvenance),
            _ => Err(format!(
                "desktop-artifacts-stage: unsupported stage {value}"
            )),
        }
    }
}

fn run_status(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|error| format!("desktop artifacts: launch {program}: {error}"))?;
    require_success(program, status)
}
fn run_output(root: &Path, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("desktop artifacts: launch {program}: {error}"))
}

fn output_text(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = run_output(root, program, args)?;
    if !output.status.success() {
        return Err(format!(
            "desktop artifacts: {program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn require_success(label: &str, status: ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("desktop artifacts: {label} failed with {status}"))
    }
}
fn env_path(name: &str) -> Result<PathBuf, String> {
    env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        format!("desktop artifacts: required environment variable {name} is missing")
    })
}

fn validate_requested_commit(root: &Path) -> Result<String, String> {
    let requested = env::var("REQUESTED_COMMIT_SHA")
        .map_err(|_| "desktop artifacts: REQUESTED_COMMIT_SHA is required".to_owned())?;
    if requested.len() != 40
        || !requested
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(
            "desktop artifacts: commit SHA must be exactly 40 lowercase hex characters".to_owned(),
        );
    }
    let actual = output_text(root, "git", &["rev-parse", "HEAD"])?;
    if actual != requested {
        return Err(format!(
            "desktop artifacts: checked-out commit {actual} != requested {requested}"
        ));
    }
    Ok(requested)
}
fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("desktop artifacts: open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("desktop artifacts: read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn recreate_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("desktop artifacts: remove {}: {error}", path.display()))?;
    }
    fs::create_dir_all(path)
        .map_err(|error| format!("desktop artifacts: create {}: {error}", path.display()))
}
fn find_files(root: &Path, predicate: &impl Fn(&Path) -> bool) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    if !root.exists() {
        return Ok(found);
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("desktop artifacts: read {}: {error}", root.display()))?
    {
        let path = entry
            .map_err(|error| format!("desktop artifacts: directory entry: {error}"))?
            .path();
        if path.is_dir() {
            found.extend(find_files(&path, predicate)?);
        } else if predicate(&path) {
            found.push(path);
        }
    }
    Ok(found)
}

fn append_env_file(name: &str, value: &str) -> Result<(), String> {
    let path = env_path(name)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("desktop artifacts: open {}: {error}", path.display()))?;
    writeln!(file, "{value}")
        .map_err(|error| format!("desktop artifacts: append {}: {error}", path.display()))
}
fn npm_command(root: &Path, args: &[&str]) -> Command {
    let mut command;
    #[cfg(windows)]
    {
        command = Command::new("cmd");
        command.arg("/C").arg("npm.cmd");
    }
    #[cfg(not(windows))]
    {
        command = Command::new("npm");
    }
    command.args(args).current_dir(root);
    command
}

fn verify_npm(root: &Path) -> Result<(), String> {
    let output = npm_command(root, &["--version"])
        .output()
        .map_err(|error| format!("desktop artifacts: launch npm: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "desktop artifacts: npm failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let actual = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if actual == EXPECTED_NPM {
        Ok(())
    } else {
        Err(format!(
            "desktop artifacts: npm version {actual} != expected {EXPECTED_NPM}"
        ))
    }
}

fn npm_ci(desktop: &Path) -> Result<(), String> {
    let status = npm_command(desktop, &["ci", "--ignore-scripts"])
        .status()
        .map_err(|error| format!("desktop artifacts: launch npm: {error}"))?;
    require_success("npm ci", status)
}

fn materialize_frontend_wasm(root: &Path) -> Result<(), String> {
    let mut args = ["check".to_owned()].into_iter();
    crate::frontend_wasm_codegen::run_cli(&mut args, root).map(|_| ())
}

#[cfg(target_os = "macos")]
fn build_tauri(desktop: &Path, target: &str, protoc: &Path) -> Result<(), String> {
    let status = Command::new("node")
        .args([
            "node_modules/@tauri-apps/cli/tauri.js",
            "build",
            "--ci",
            "--target",
            target,
        ])
        .env("PROTOC", protoc)
        .current_dir(desktop)
        .status()
        .map_err(|error| format!("desktop artifacts: launch Tauri build: {error}"))?;
    require_success("Tauri build", status)
}
fn generate_sbom(root: &Path, artifact_dir: &Path, name: &str) -> Result<(), String> {
    let sbom = artifact_dir.join(name);
    let output_arg = format!("spdx-json={}", sbom.display());
    run_status(
        root,
        "syft",
        &[
            "scan",
            "dir:.",
            "--exclude",
            "./target/**",
            "--exclude",
            "./apps/kaspa-gateway-desktop/node_modules/**",
            "-o",
            &output_arg,
        ],
    )?;
    if fs::metadata(&sbom).map(|meta| meta.len()).unwrap_or(0) == 0 {
        return Err(format!(
            "desktop artifacts: SBOM {} is empty",
            sbom.display()
        ));
    }
    let value: Value = serde_json::from_slice(
        &fs::read(&sbom)
            .map_err(|error| format!("desktop artifacts: read {}: {error}", sbom.display()))?,
    )
    .map_err(|error| format!("desktop artifacts: parse {}: {error}", sbom.display()))?;
    if value.get("spdxVersion").and_then(Value::as_str) != Some("SPDX-2.3") {
        return Err("desktop artifacts: SBOM must be SPDX-2.3".to_owned());
    }
    Ok(())
}
fn preserve_attestation(artifact_dir: &Path, destination_name: &str) -> Result<(), String> {
    let source = env_path("ATTESTATION_BUNDLE")?;
    if fs::metadata(&source).map(|meta| meta.len()).unwrap_or(0) == 0 {
        return Err("desktop artifacts: attestation bundle is missing or empty".to_owned());
    }
    fs::create_dir_all(artifact_dir).map_err(|error| {
        format!(
            "desktop artifacts: create {}: {error}",
            artifact_dir.display()
        )
    })?;
    let destination = artifact_dir.join(destination_name);
    fs::copy(&source, &destination).map_err(|error| {
        format!(
            "desktop artifacts: copy {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    if fs::metadata(&destination)
        .map(|meta| meta.len())
        .unwrap_or(0)
        == 0
    {
        return Err("desktop artifacts: preserved attestation bundle is empty".to_owned());
    }
    Ok(())
}
#[cfg(windows)]
fn install_protoc_windows(root: &Path) -> Result<PathBuf, String> {
    let temp = env_path("RUNNER_TEMP")?;
    let archive = temp.join(format!("protoc-{PROTOC_VERSION}-win64.zip"));
    let install = temp.join(format!("protoc-{PROTOC_VERSION}-win64"));
    if install.exists() {
        fs::remove_dir_all(&install)
            .map_err(|error| format!("desktop artifacts: remove {}: {error}", install.display()))?;
    }
    let url = format!(
        "https://github.com/protocolbuffers/protobuf/releases/download/v{PROTOC_VERSION}/protoc-{PROTOC_VERSION}-win64.zip"
    );
    let archive_text = archive.to_string_lossy().into_owned();
    run_status(
        root,
        "curl.exe",
        &[
            "--proto",
            "=https",
            "--tlsv1.2",
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--output",
            &archive_text,
            &url,
        ],
    )?;
    let actual = sha256_file(&archive)?;
    if actual != WINDOWS_PROTOC_SHA256 {
        return Err(format!(
            "desktop artifacts: Windows protoc checksum mismatch: {actual}"
        ));
    }
    fs::create_dir_all(&install)
        .map_err(|error| format!("desktop artifacts: create {}: {error}", install.display()))?;
    let install_text = install.to_string_lossy().into_owned();
    run_status(
        root,
        "tar.exe",
        &["-xf", &archive_text, "-C", &install_text],
    )?;
    let bin = install.join("bin");
    append_env_file("GITHUB_PATH", &bin.to_string_lossy())?;
    let protoc = bin.join("protoc.exe");
    if !protoc.is_file() {
        return Err("desktop artifacts: extracted Windows protoc is missing".to_owned());
    }
    Ok(protoc)
}

#[cfg(windows)]
fn windows_signature_status(_root: &Path, installer: &Path) -> Result<String, String> {
    let bytes = fs::read(installer).map_err(|error| {
        format!("desktop artifacts: read PE for Authenticode inspection: {error}")
    })?;
    let u16_at = |offset: usize| -> Result<u16, String> {
        let slice = bytes
            .get(offset..offset + 2)
            .ok_or_else(|| "desktop artifacts: truncated PE while reading u16".to_owned())?;
        Ok(u16::from_le_bytes([slice[0], slice[1]]))
    };
    let u32_at = |offset: usize| -> Result<u32, String> {
        let slice = bytes
            .get(offset..offset + 4)
            .ok_or_else(|| "desktop artifacts: truncated PE while reading u32".to_owned())?;
        Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    };

    if bytes.get(..2) != Some(b"MZ") {
        return Err("desktop artifacts: installer is not a DOS/PE executable".to_owned());
    }
    let pe_offset = usize::try_from(u32_at(0x3c)?)
        .map_err(|_| "desktop artifacts: PE header offset overflow".to_owned())?;
    if bytes.get(pe_offset..pe_offset + 4) != Some(b"PE\0\0") {
        return Err("desktop artifacts: invalid PE signature".to_owned());
    }

    let optional = pe_offset
        .checked_add(24)
        .ok_or_else(|| "desktop artifacts: PE optional-header offset overflow".to_owned())?;
    let data_directories = match u16_at(optional)? {
        0x10b => optional + 96,
        0x20b => optional + 112,
        magic => {
            return Err(format!(
                "desktop artifacts: unsupported PE optional-header magic 0x{magic:04x}"
            ));
        }
    };
    let security = data_directories
        .checked_add(4 * 8)
        .ok_or_else(|| "desktop artifacts: PE security-directory offset overflow".to_owned())?;
    let certificate_table_offset = u32_at(security)?;
    let certificate_table_size = u32_at(security + 4)?;

    if certificate_table_offset == 0 && certificate_table_size == 0 {
        Ok("NotSigned".to_owned())
    } else if certificate_table_offset != 0 && certificate_table_size != 0 {
        Ok("SignedPresent".to_owned())
    } else {
        Err("desktop artifacts: malformed PE certificate table entry".to_owned())
    }
}

#[cfg(windows)]
fn verify_windows_runtime(executable: &Path, report: &Path) -> Result<(), String> {
    let mut args = vec![
        "--executable".to_owned(),
        executable.to_string_lossy().into_owned(),
        "--report-path".to_owned(),
        report.to_string_lossy().into_owned(),
    ]
    .into_iter();
    crate::windows_runtime_dependencies::run_cli(&mut args).map(|_| ())
}

#[cfg(windows)]
fn run_windows(root: &Path) -> Result<(), String> {
    let requested = validate_requested_commit(root)?;
    verify_npm(root)?;
    materialize_frontend_wasm(root)?;
    let protoc = install_protoc_windows(root)?;
    let llvm_bin = PathBuf::from(env::var_os("ProgramFiles").unwrap_or_default()).join("LLVM/bin");
    if !llvm_bin.join("libclang.dll").is_file() {
        return Err(
            "desktop artifacts: hosted runner LLVM/libclang installation is missing".to_owned(),
        );
    }
    let llvm_ar = llvm_bin.join("llvm-ar.exe");
    let ar = llvm_bin.join("ar.exe");
    if llvm_ar.is_file() && !ar.is_file() {
        fs::copy(&llvm_ar, &ar)
            .map_err(|error| format!("desktop artifacts: create LLVM ar alias: {error}"))?;
    }

    let desktop = root.join("apps/kaspa-gateway-desktop");
    npm_ci(&desktop)?;
    let build_status = Command::new("node")
        .args([
            "node_modules/@tauri-apps/cli/tauri.js",
            "build",
            "--ci",
            "--target",
            "x86_64-pc-windows-msvc",
        ])
        .env("PROTOC", &protoc)
        .env("LIBCLANG_PATH", &llvm_bin)
        .current_dir(&desktop)
        .status()
        .map_err(|error| format!("desktop artifacts: launch Windows Tauri build: {error}"))?;
    require_success("Windows Tauri build", build_status)?;
    let temp = env_path("RUNNER_TEMP")?;
    let artifact_dir = temp.join("kaspa-gateway-windows-artifact");
    let install_dir = temp.join("kaspa-gateway-installed");
    let data_dir = temp.join("kaspa-gateway-data");
    recreate_dir(&artifact_dir)?;
    recreate_dir(&install_dir)?;
    recreate_dir(&data_dir)?;

    let bundle_root = root.join("target/x86_64-pc-windows-msvc/release/bundle/nsis");
    let installers = find_files(&bundle_root, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("exe")
    })?;
    if installers.len() != 1 || fs::metadata(&installers[0]).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(format!(
            "desktop artifacts: expected exactly one non-empty NSIS installer; found {}",
            installers.len()
        ));
    }
    let raw_exe = root.join("target/x86_64-pc-windows-msvc/release/kaspa-gateway-desktop.exe");
    if !raw_exe.is_file() {
        return Err(format!(
            "desktop artifacts: raw executable missing: {}",
            raw_exe.display()
        ));
    }
    verify_windows_runtime(
        &raw_exe,
        &artifact_dir.join("raw-runtime-dependencies.json"),
    )?;
    let installer = artifact_dir.join("KaspaGateway-windows-x64-nsis.exe");
    let raw_artifact = artifact_dir.join("kaspa-gateway-desktop-windows-x64.exe");
    fs::copy(&installers[0], &installer)
        .map_err(|error| format!("desktop artifacts: copy installer: {error}"))?;
    fs::copy(&raw_exe, &raw_artifact)
        .map_err(|error| format!("desktop artifacts: copy raw executable: {error}"))?;

    let signature = windows_signature_status(root, &installer)?;
    if signature != "NotSigned" {
        return Err(format!(
            "desktop artifacts: expected unsigned internal installer; Authenticode={signature}"
        ));
    }

    let install_arg = format!("/D={}", install_dir.display());
    let install_status = Command::new(&installer)
        .args(["/S", &install_arg])
        .status()
        .map_err(|error| format!("desktop artifacts: launch NSIS installer: {error}"))?;
    require_success("silent NSIS install", install_status)?;
    let installed = find_files(&install_dir, &|path| {
        path.file_name().and_then(|value| value.to_str()) == Some("kaspa-gateway-desktop.exe")
    })?;
    if installed.len() != 1 {
        return Err(format!(
            "desktop artifacts: expected exactly one installed executable; found {}",
            installed.len()
        ));
    }
    let installed_hash = sha256_file(&installed[0])?;
    verify_windows_runtime(
        &installed[0],
        &artifact_dir.join("installed-runtime-dependencies.json"),
    )?;

    let mut child = Command::new(&installed[0])
        .env("KASPA_GATEWAY_DATA_DIR", &data_dir)
        .spawn()
        .map_err(|error| format!("desktop artifacts: launch installed application: {error}"))?;
    thread::sleep(Duration::from_secs(15));
    if let Some(status) = child
        .try_wait()
        .map_err(|error| format!("desktop artifacts: inspect installed application: {error}"))?
    {
        return Err(format!(
            "desktop artifacts: installed application exited during smoke window: {status}"
        ));
    }
    child
        .kill()
        .map_err(|error| format!("desktop artifacts: stop application: {error}"))?;
    let _ = child.wait();
    let uninstallers = find_files(&install_dir, &|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                let lower = name.to_ascii_lowercase();
                lower == "uninstall.exe" || (lower.starts_with("unins") && lower.ends_with(".exe"))
            })
    })?;
    if uninstallers.len() != 1 {
        return Err(format!(
            "desktop artifacts: expected exactly one NSIS uninstaller; found {}",
            uninstallers.len()
        ));
    }
    let uninstall_status = Command::new(&uninstallers[0])
        .arg("/S")
        .status()
        .map_err(|error| format!("desktop artifacts: launch NSIS uninstaller: {error}"))?;
    require_success("silent NSIS uninstall", uninstall_status)?;
    thread::sleep(Duration::from_secs(3));
    if installed[0].is_file() {
        return Err("desktop artifacts: installed executable remains after uninstall".to_owned());
    }
    let installer_hash = sha256_file(&installer)?;
    let raw_hash = sha256_file(&raw_artifact)?;
    fs::write(
        artifact_dir.join("SHA256SUMS"),
        format!(
            "{installer_hash}  KaspaGateway-windows-x64-nsis.exe\n{raw_hash}  kaspa-gateway-desktop-windows-x64.exe\n"
        ),
    )
    .map_err(|error| format!("desktop artifacts: write Windows SHA256SUMS: {error}"))?;
    fs::write(
        artifact_dir.join("WINDOWS_INSTALLER_SMOKE.txt"),
        format!(
            "REQUESTED_COMMIT_SHA={requested}\nWINDOWS_PRIMARY_FORMAT=NSIS_EXE\nWINDOWS_ARCHITECTURE=X86_64\nWINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED\nWINDOWS_AUTHENTICODE_STATUS={signature}\nWINDOWS_INSTALLED_EXE_SHA256={installed_hash}\nWINDOWS_INSTALLER_SMOKE=PASS\nWINDOWS_LAUNCH_OBSERVATION_SECONDS=15\nWINDOWS_UNINSTALL_SMOKE=PASS\nTESTNET13_LIVE_SMOKE=NOT_RUN_EXPERIMENTAL\n"
        ),
    )
    .map_err(|error| format!("desktop artifacts: write Windows smoke report: {error}"))?;
    generate_sbom(root, &artifact_dir, "WINDOWS_SBOM.spdx.json")?;
    Ok(())
}
#[cfg(target_os = "macos")]
fn install_protoc_macos(root: &Path) -> Result<PathBuf, String> {
    let temp = env_path("RUNNER_TEMP")?;
    let archive = temp.join(format!("protoc-{PROTOC_VERSION}-osx-universal_binary.zip"));
    let install = temp.join(format!("protoc-{PROTOC_VERSION}-osx-universal_binary"));
    if install.exists() {
        fs::remove_dir_all(&install)
            .map_err(|error| format!("desktop artifacts: remove {}: {error}", install.display()))?;
    }
    let url = format!(
        "https://github.com/protocolbuffers/protobuf/releases/download/v{PROTOC_VERSION}/protoc-{PROTOC_VERSION}-osx-universal_binary.zip"
    );
    let archive_text = archive.to_string_lossy().into_owned();
    run_status(
        root,
        "curl",
        &[
            "--proto",
            "=https",
            "--tlsv1.2",
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--output",
            &archive_text,
            &url,
        ],
    )?;
    let actual = sha256_file(&archive)?;
    if actual != MACOS_PROTOC_SHA256 {
        return Err(format!(
            "desktop artifacts: macOS protoc checksum mismatch: {actual}"
        ));
    }
    fs::create_dir_all(&install)
        .map_err(|error| format!("desktop artifacts: create {}: {error}", install.display()))?;
    let archive_arg = archive.to_string_lossy().into_owned();
    let install_arg = install.to_string_lossy().into_owned();
    run_status(root, "ditto", &["-x", "-k", &archive_arg, &install_arg])?;
    let bin = install.join("bin");
    append_env_file("GITHUB_PATH", &bin.to_string_lossy())?;
    let protoc = bin.join("protoc");
    if !protoc.is_file() {
        return Err("desktop artifacts: extracted macOS protoc is missing".to_owned());
    }
    Ok(protoc)
}

#[cfg(target_os = "macos")]
fn mounted_apps(mount: &Path) -> Result<Vec<PathBuf>, String> {
    let mut apps = Vec::new();
    for entry in fs::read_dir(mount)
        .map_err(|error| format!("desktop artifacts: read {}: {error}", mount.display()))?
    {
        let path = entry
            .map_err(|error| format!("desktop artifacts: mount entry: {error}"))?
            .path();
        if path.is_dir() && path.extension().and_then(|value| value.to_str()) == Some("app") {
            apps.push(path);
        }
    }
    Ok(apps)
}
#[cfg(target_os = "macos")]
fn run_macos(root: &Path) -> Result<(), String> {
    let requested = validate_requested_commit(root)?;
    verify_npm(root)?;
    materialize_frontend_wasm(root)?;
    let protoc = install_protoc_macos(root)?;
    run_status(root, "xcodebuild", &["-version"])?;
    run_status(root, "xcrun", &["--find", "lipo"])?;

    let desktop = root.join("apps/kaspa-gateway-desktop");
    npm_ci(&desktop)?;
    build_tauri(&desktop, "universal-apple-darwin", &protoc)?;

    let temp = env_path("RUNNER_TEMP")?;
    let artifact_dir = temp.join("kaspa-gateway-macos-artifact");
    let mount_dir = temp.join("kaspa-gateway-dmg-mount");
    let data_dir = temp.join("kaspa-gateway-data");
    recreate_dir(&artifact_dir)?;
    recreate_dir(&mount_dir)?;
    recreate_dir(&data_dir)?;

    let bundle_root = root.join("target/universal-apple-darwin/release/bundle/dmg");
    let dmgs = find_files(&bundle_root, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("dmg")
    })?;
    if dmgs.len() != 1 || fs::metadata(&dmgs[0]).map(|m| m.len()).unwrap_or(0) == 0 {
        return Err(format!(
            "desktop artifacts: expected exactly one non-empty Universal DMG; found {}",
            dmgs.len()
        ));
    }
    let dmg = artifact_dir.join("KaspaGateway-macos-universal.dmg");
    let app_zip = artifact_dir.join("KaspaGateway-macos-universal-app.zip");
    let exported_app = artifact_dir.join("KaspaGateway-macos-universal.app");
    fs::copy(&dmgs[0], &dmg).map_err(|error| format!("desktop artifacts: copy DMG: {error}"))?;

    let dmg_arg = dmg.to_string_lossy().into_owned();
    let mount_arg = mount_dir.to_string_lossy().into_owned();
    run_status(
        root,
        "hdiutil",
        &[
            "attach",
            &dmg_arg,
            "-readonly",
            "-nobrowse",
            "-mountpoint",
            &mount_arg,
        ],
    )?;

    let qualification = (|| -> Result<(String, String), String> {
        let apps = mounted_apps(&mount_dir)?;
        if apps.len() != 1 {
            return Err(format!(
                "desktop artifacts: expected exactly one app in DMG; found {}",
                apps.len()
            ));
        }
        let mounted_app = &apps[0];
        let plist = mounted_app.join("Contents/Info.plist");
        let plist_arg = plist.to_string_lossy().into_owned();
        let executable_name = output_text(
            root,
            "/usr/libexec/PlistBuddy",
            &["-c", "Print :CFBundleExecutable", &plist_arg],
        )?;
        let executable = mounted_app.join("Contents/MacOS").join(&executable_name);
        if !executable.is_file() {
            return Err(format!(
                "desktop artifacts: mounted app executable missing: {}",
                executable.display()
            ));
        }

        if exported_app.exists() {
            fs::remove_dir_all(&exported_app)
                .map_err(|error| format!("desktop artifacts: remove exported app: {error}"))?;
        }
        let mounted_arg = mounted_app.to_string_lossy().into_owned();
        let exported_arg = exported_app.to_string_lossy().into_owned();
        run_status(root, "ditto", &[&mounted_arg, &exported_arg])?;
        let zip_arg = app_zip.to_string_lossy().into_owned();
        run_status(
            root,
            "ditto",
            &[
                "-c",
                "-k",
                "--sequesterRsrc",
                "--keepParent",
                &exported_arg,
                &zip_arg,
            ],
        )?;
        let exe_arg = executable.to_string_lossy().into_owned();
        let architectures = output_text(root, "lipo", &["-archs", &exe_arg])?;
        let architecture_words = architectures.split_whitespace().collect::<Vec<_>>();
        if !architecture_words.contains(&"arm64") || !architecture_words.contains(&"x86_64") {
            return Err(format!(
                "desktop artifacts: universal architecture proof failed: {architectures}"
            ));
        }
        let app_arg = mounted_app.to_string_lossy().into_owned();
        run_status(
            root,
            "codesign",
            &["--verify", "--deep", "--strict", "--verbose=2", &app_arg],
        )?;
        let details_output = run_output(root, "codesign", &["-dv", "--verbose=4", &app_arg])?;
        if !details_output.status.success() {
            return Err("desktop artifacts: codesign detail inspection failed".to_owned());
        }
        let details = format!(
            "{}{}",
            String::from_utf8_lossy(&details_output.stdout),
            String::from_utf8_lossy(&details_output.stderr)
        );
        let signing = if details.contains("Signature=adhoc") || details.contains("adhoc") {
            "AD_HOC"
        } else if details
            .lines()
            .any(|line| line.trim_start().starts_with("Authority="))
        {
            "DEVELOPER_ID"
        } else {
            return Err("desktop artifacts: unsupported macOS signing mode".to_owned());
        };
        let stdout_path = artifact_dir.join("macos-launch.stdout.log");
        let stderr_path = artifact_dir.join("macos-launch.stderr.log");
        let stdout = File::create(&stdout_path)
            .map_err(|error| format!("desktop artifacts: create launch stdout: {error}"))?;
        let stderr = File::create(&stderr_path)
            .map_err(|error| format!("desktop artifacts: create launch stderr: {error}"))?;
        let mut child = Command::new(&executable)
            .env("KASPA_GATEWAY_DATA_DIR", &data_dir)
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|error| format!("desktop artifacts: launch mounted app: {error}"))?;
        thread::sleep(Duration::from_secs(15));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("desktop artifacts: inspect mounted app: {error}"))?
        {
            return Err(format!(
                "desktop artifacts: mounted app exited during smoke window: {status}"
            ));
        }
        let pid = child.id().to_string();
        let _ = run_status(root, "kill", &["-TERM", &pid]);
        let _ = child.wait();
        Ok((architectures, format!("{signing}\n{details}")))
    })();
    let detach_result = run_status(root, "hdiutil", &["detach", &mount_arg, "-quiet"]);
    let (architectures, signing_and_details) = qualification?;
    detach_result?;
    let mut lines = signing_and_details.splitn(2, '\n');
    let signing = lines.next().unwrap_or("UNKNOWN");
    let details = lines.next().unwrap_or_default();

    let dmg_hash = sha256_file(&dmg)?;
    let app_hash = sha256_file(&app_zip)?;
    fs::write(
        artifact_dir.join("SHA256SUMS"),
        format!(
            "{dmg_hash}  KaspaGateway-macos-universal.dmg\n{app_hash}  KaspaGateway-macos-universal-app.zip\n"
        ),
    )
    .map_err(|error| format!("desktop artifacts: write macOS SHA256SUMS: {error}"))?;
    fs::write(
        artifact_dir.join("MACOS_CODESIGN_DETAILS.txt"),
        format!("{details}\n"),
    )
    .map_err(|error| format!("desktop artifacts: write codesign details: {error}"))?;
    fs::write(
        artifact_dir.join("MACOS_DMG_SMOKE.txt"),
        format!(
            "REQUESTED_COMMIT_SHA={requested}\nMACOS_PRIMARY_FORMAT=DMG\nMACOS_ARCHITECTURE_MODE=UNIVERSAL\nMACOS_CODE_SIGNING={signing}\nMACOS_NOTARIZATION=NOT_CONFIGURED\nMACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64\nMACOS_EXECUTABLE_ARCHITECTURES={architectures}\nMACOS_CODESIGN_VERIFY=PASS\nMACOS_DMG_SMOKE=PASS\nMACOS_LAUNCH_OBSERVATION_SECONDS=15\nMACOS_INTERACTIVE_UI_VALIDATION=NOT_PERFORMED_HOSTED_RUNNER\nTESTNET13_LIVE_SMOKE=NOT_RUN_EXPERIMENTAL\n"
        ),
    )
    .map_err(|error| format!("desktop artifacts: write macOS smoke report: {error}"))?;
    generate_sbom(root, &artifact_dir, "MACOS_SBOM.spdx.json")?;
    Ok(())
}

pub fn run_stage(root: &Path, value: &str) -> Result<String, String> {
    let stage = Stage::parse(value)?;
    match stage {
        Stage::Windows => {
            #[cfg(windows)]
            run_windows(root)?;
            #[cfg(not(windows))]
            return Err("desktop-artifacts-stage windows requires Windows".to_owned());
        }
        Stage::Macos => {
            #[cfg(target_os = "macos")]
            run_macos(root)?;
            #[cfg(not(target_os = "macos"))]
            return Err("desktop-artifacts-stage macos requires macOS".to_owned());
        }
        Stage::PreserveWindowsSbom => preserve_attestation(
            &env_path("RUNNER_TEMP")?.join("kaspa-gateway-windows-artifact"),
            "WINDOWS_SBOM_ATTESTATION.sigstore.json",
        )?,
        Stage::PreserveWindowsProvenance => preserve_attestation(
            &env_path("RUNNER_TEMP")?.join("kaspa-gateway-windows-artifact"),
            "WINDOWS_BUILD_PROVENANCE.sigstore.json",
        )?,
        Stage::PreserveMacosSbom => preserve_attestation(
            &env_path("RUNNER_TEMP")?.join("kaspa-gateway-macos-artifact"),
            "MACOS_SBOM_ATTESTATION.sigstore.json",
        )?,
        Stage::PreserveMacosProvenance => preserve_attestation(
            &env_path("RUNNER_TEMP")?.join("kaspa-gateway-macos-artifact"),
            "MACOS_BUILD_PROVENANCE.sigstore.json",
        )?,
    }
    Ok(format!("DESKTOP_ARTIFACT_STAGE={value} PASS"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_parser_is_fail_closed() {
        assert_eq!(Stage::parse("windows"), Ok(Stage::Windows));
        assert_eq!(Stage::parse("macos"), Ok(Stage::Macos));
        assert!(Stage::parse("powershell").is_err());
        assert!(Stage::parse("bash").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn windows_signature_status_reads_pe_certificate_directory_without_powershell() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("fixture.exe");
        let mut pe = vec![0u8; 512];
        pe[0..2].copy_from_slice(b"MZ");
        pe[0x3c..0x40].copy_from_slice(&(0x80u32).to_le_bytes());
        pe[0x80..0x84].copy_from_slice(b"PE\0\0");
        let optional = 0x80 + 24;
        pe[optional..optional + 2].copy_from_slice(&(0x20bu16).to_le_bytes());
        let security = optional + 112 + (4 * 8);

        fs::write(&path, &pe).expect("write unsigned fixture");
        assert_eq!(
            windows_signature_status(temp.path(), &path).as_deref(),
            Ok("NotSigned")
        );

        pe[security..security + 4].copy_from_slice(&(400u32).to_le_bytes());
        pe[security + 4..security + 8].copy_from_slice(&(64u32).to_le_bytes());
        fs::write(&path, &pe).expect("write signed fixture");
        assert_eq!(
            windows_signature_status(temp.path(), &path).as_deref(),
            Ok("SignedPresent")
        );
    }

    #[test]
    fn npm_command_is_platform_correct() {
        let command = npm_command(Path::new("."), &["--version"]);
        let program = command.get_program().to_string_lossy().into_owned();
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        #[cfg(windows)]
        {
            assert_eq!(program, "cmd");
            assert_eq!(args, vec!["/C", "npm.cmd", "--version"]);
        }
        #[cfg(not(windows))]
        {
            assert_eq!(program, "npm");
            assert_eq!(args, vec!["--version"]);
        }
    }
}
