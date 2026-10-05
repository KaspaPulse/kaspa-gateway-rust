use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

const CONFIRMATION: &str = "STAGE_DESKTOP_RELEASE_DRAFT";
const SIGNER_PREDICATE: &str = "https://slsa.dev/provenance/v1";
const SBOM_PREDICATE: &str = "https://spdx.dev/Document/v2.3";
const MAX_GH_JSON: usize = 16 * 1024 * 1024;

#[derive(Debug)]
struct Context {
    repository: String,
    github_ref: String,
    github_sha: String,
    version: String,
    commit_sha: String,
    artifact_run_id: String,
    confirmation: String,
    signer_workflow: String,
    runner_temp: PathBuf,
    github_output: Option<PathBuf>,
}

impl Context {
    fn from_env() -> Result<Self, String> {
        Ok(Self {
            repository: required_env("GITHUB_REPOSITORY")?,
            github_ref: required_env("GITHUB_REF")?,
            github_sha: required_env("GITHUB_SHA")?,
            version: required_env("REQUESTED_VERSION")?,
            commit_sha: required_env("REQUESTED_COMMIT_SHA")?,
            artifact_run_id: required_env("ARTIFACT_RUN_ID")?,
            confirmation: required_env("RELEASE_CONFIRMATION")?,
            signer_workflow: required_env("SIGNER_WORKFLOW")?,
            runner_temp: PathBuf::from(required_env("RUNNER_TEMP")?),
            github_output: env::var_os("GITHUB_OUTPUT").map(PathBuf::from),
        })
    }
}

fn required_env(name: &str) -> Result<String, String> {
    let value = env::var(name)
        .map_err(|_| format!("desktop release draft stage: missing environment variable {name}"))?;
    if value.trim().is_empty() {
        Err(format!(
            "desktop release draft stage: empty environment variable {name}"
        ))
    } else {
        Ok(value)
    }
}

fn is_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

fn is_lower_hex40(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn is_lower_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn is_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

fn command_output(root: Option<&Path>, program: &str, args: &[String]) -> Result<Output, String> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(root) = root {
        command.current_dir(root);
    }
    command.output().map_err(|error| {
        format!("desktop release draft stage: failed to launch {program}: {error}")
    })
}

fn checked_output(root: Option<&Path>, program: &str, args: &[String]) -> Result<Output, String> {
    let output = command_output(root, program, args)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "desktop release draft stage: {program} failed ({})\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn checked_status(root: Option<&Path>, program: &str, args: &[String]) -> Result<(), String> {
    checked_output(root, program, args).map(|_| ())
}

fn gh_json(args: &[String]) -> Result<Value, String> {
    let output = checked_output(None, "gh", args)?;
    if output.stdout.len() > MAX_GH_JSON {
        return Err(
            "desktop release draft stage: gh JSON response exceeds bounded size".to_owned(),
        );
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("desktop release draft stage: invalid gh JSON: {error}"))
}

fn git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    let args = args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let output = checked_output(Some(root), "git", &args)?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("desktop release draft stage: git output was not UTF-8: {error}"))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "desktop release draft stage: read {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "desktop release draft stage: parse {}: {error}",
            path.display()
        )
    })
}

fn cargo_package_version(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && let Some(rest) = line.strip_prefix("version") {
            let rest = rest.trim_start().strip_prefix('=')?.trim();
            if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
                return Some(rest[1..rest.len() - 1].to_owned());
            }
        }
    }
    None
}

fn validate_request(ctx: &Context) -> Result<(), String> {
    if ctx.github_ref != "refs/heads/main" {
        return Err(
            "desktop release draft stage: workflow must run from refs/heads/main".to_owned(),
        );
    }
    if !is_version(&ctx.version) {
        return Err("desktop release draft stage: version must be stable X.Y.Z".to_owned());
    }
    if !is_lower_hex40(&ctx.commit_sha) {
        return Err(
            "desktop release draft stage: commit SHA must be 40 lowercase hex characters"
                .to_owned(),
        );
    }
    if !is_digits(&ctx.artifact_run_id) {
        return Err("desktop release draft stage: artifact run ID must be numeric".to_owned());
    }
    if ctx.confirmation != CONFIRMATION {
        return Err("desktop release draft stage: explicit confirmation is required".to_owned());
    }
    if ctx.github_sha != ctx.commit_sha {
        return Err(
            "desktop release draft stage: dispatch SHA does not match requested commit".to_owned(),
        );
    }
    Ok(())
}

fn release_pages(repository: &str) -> Result<Value, String> {
    gh_json(&[
        "api".to_owned(),
        "--paginate".to_owned(),
        "--slurp".to_owned(),
        format!("repos/{repository}/releases?per_page=100"),
    ])
}

fn releases(pages: &Value) -> impl Iterator<Item = &Value> {
    pages
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_array)
        .flatten()
}

fn matching_draft<'a>(pages: &'a Value, tag: &str, target: &str) -> Vec<&'a Value> {
    releases(pages)
        .filter(|release| {
            release.get("draft").and_then(Value::as_bool) == Some(true)
                && release.get("tag_name").and_then(Value::as_str) == Some(tag)
                && release.get("target_commitish").and_then(Value::as_str) == Some(target)
        })
        .collect()
}

fn validate_remote_preconditions(ctx: &Context) -> Result<(), String> {
    let live = gh_json(&[
        "api".to_owned(),
        format!("repos/{}/commits/main", ctx.repository),
    ])?;
    if live.get("sha").and_then(Value::as_str) != Some(ctx.commit_sha.as_str()) {
        return Err("desktop release draft stage: requested commit is not current main".to_owned());
    }
    let tag = format!("desktop-v{}", ctx.version);
    let refs = gh_json(&[
        "api".to_owned(),
        format!("repos/{}/git/matching-refs/tags/{tag}", ctx.repository),
    ])?;
    if !refs.as_array().is_some_and(Vec::is_empty) {
        return Err(format!(
            "desktop release draft stage: tag already exists: {tag}"
        ));
    }
    let pages = release_pages(&ctx.repository)?;
    if releases(&pages).any(|r| r.get("tag_name").and_then(Value::as_str) == Some(tag.as_str())) {
        return Err(format!(
            "desktop release draft stage: release already exists: {tag}"
        ));
    }
    Ok(())
}

fn validate_checkout_and_version(root: &Path, ctx: &Context) -> Result<(), String> {
    if git_text(root, &["rev-parse", "HEAD"])? != ctx.commit_sha {
        return Err("desktop release draft stage: checkout HEAD mismatch".to_owned());
    }
    let package = read_json(&root.join("apps/kaspa-gateway-desktop/package.json"))?;
    let tauri = read_json(&root.join("apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json"))?;
    let cargo_text =
        fs::read_to_string(root.join("apps/kaspa-gateway-desktop/src-tauri/Cargo.toml"))
            .map_err(|error| format!("desktop release draft stage: read Cargo.toml: {error}"))?;
    let cargo_version = cargo_package_version(&cargo_text);
    if package.get("version").and_then(Value::as_str) != Some(ctx.version.as_str())
        || tauri.get("version").and_then(Value::as_str) != Some(ctx.version.as_str())
        || cargo_version.as_deref() != Some(ctx.version.as_str())
    {
        return Err(
            "desktop release draft stage: desktop version files do not match requested version"
                .to_owned(),
        );
    }
    Ok(())
}

fn validate_artifact_run(ctx: &Context) -> Result<(), String> {
    let run = gh_json(&[
        "api".to_owned(),
        format!(
            "repos/{}/actions/runs/{}",
            ctx.repository, ctx.artifact_run_id
        ),
    ])?;
    for (key, expected) in [
        ("name", "Desktop Artifacts"),
        ("path", ".github/workflows/desktop-artifacts.yml"),
        ("event", "workflow_dispatch"),
        ("status", "completed"),
        ("conclusion", "success"),
        ("head_sha", ctx.commit_sha.as_str()),
    ] {
        if run.get(key).and_then(Value::as_str) != Some(expected) {
            return Err(format!(
                "desktop release draft stage: artifact run {key} mismatch"
            ));
        }
    }
    let artifacts = gh_json(&[
        "api".to_owned(),
        format!(
            "repos/{}/actions/runs/{}/artifacts?per_page=100",
            ctx.repository, ctx.artifact_run_id
        ),
    ])?;
    let list = artifacts
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "desktop release draft stage: artifacts response omitted artifacts".to_owned()
        })?;
    for name in [
        format!("kaspa-gateway-windows-x64-{}", ctx.commit_sha),
        format!("kaspa-gateway-macos-universal-{}", ctx.commit_sha),
    ] {
        let count = list
            .iter()
            .filter(|item| {
                item.get("name").and_then(Value::as_str) == Some(name.as_str())
                    && item.get("expired").and_then(Value::as_bool) == Some(false)
                    && item
                        .get("size_in_bytes")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        > 0
            })
            .count();
        if count != 1 {
            return Err(format!(
                "desktop release draft stage: expected one qualified artifact named {name}"
            ));
        }
    }
    Ok(())
}

fn reset_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| {
            format!(
                "desktop release draft stage: remove {}: {error}",
                path.display()
            )
        })?;
    }
    fs::create_dir_all(path).map_err(|error| {
        format!(
            "desktop release draft stage: create {}: {error}",
            path.display()
        )
    })
}

fn download_artifacts(ctx: &Context, root: &Path) -> Result<(PathBuf, PathBuf), String> {
    let win = root.join("windows");
    let mac = root.join("macos");
    reset_dir(&win)?;
    reset_dir(&mac)?;
    for (name, destination) in [
        (
            format!("kaspa-gateway-windows-x64-{}", ctx.commit_sha),
            &win,
        ),
        (
            format!("kaspa-gateway-macos-universal-{}", ctx.commit_sha),
            &mac,
        ),
    ] {
        checked_status(
            None,
            "gh",
            &[
                "run".to_owned(),
                "download".to_owned(),
                ctx.artifact_run_id.clone(),
                "-R".to_owned(),
                ctx.repository.clone(),
                "-n".to_owned(),
                name,
                "-D".to_owned(),
                destination.to_string_lossy().into_owned(),
            ],
        )?;
    }
    Ok((win, mac))
}

fn sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "desktop release draft stage: read {} for hash: {error}",
            path.display()
        )
    })?;
    let mut digest = Sha256::new();
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

fn verify_checksum_manifest(dir: &Path) -> Result<(), String> {
    let manifest = fs::read_to_string(dir.join("SHA256SUMS"))
        .map_err(|error| format!("desktop release draft stage: read SHA256SUMS: {error}"))?;
    let mut seen = 0usize;
    for line in manifest.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        let expected = fields
            .next()
            .ok_or_else(|| "desktop release draft stage: malformed SHA256SUMS".to_owned())?;
        let name = fields
            .next()
            .ok_or_else(|| "desktop release draft stage: malformed SHA256SUMS".to_owned())?
            .trim_start_matches('*');
        if fields.next().is_some() || expected.len() != 64 || name.is_empty() {
            return Err("desktop release draft stage: malformed SHA256SUMS entry".to_owned());
        }
        if !expected.eq_ignore_ascii_case(&sha256(&dir.join(name))?) {
            return Err(format!(
                "desktop release draft stage: checksum mismatch for {name}"
            ));
        }
        seen += 1;
    }
    if seen == 0 {
        return Err("desktop release draft stage: empty SHA256SUMS".to_owned());
    }
    Ok(())
}

fn require_nonempty(paths: &[&Path]) -> Result<(), String> {
    for path in paths {
        let metadata = fs::metadata(path).map_err(|error| {
            format!(
                "desktop release draft stage: required file {}: {error}",
                path.display()
            )
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(format!(
                "desktop release draft stage: required file empty: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn require_markers(path: &Path, markers: &[String]) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "desktop release draft stage: read {}: {error}",
            path.display()
        )
    })?;
    for marker in markers {
        if !text.contains(marker) {
            return Err(format!(
                "desktop release draft stage: {} omitted marker {marker}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn require_lower_hex64_marker(path: &Path, key: &str) -> Result<String, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "desktop release draft stage: read {}: {error}",
            path.display()
        )
    })?;
    let prefix = format!("{key}=");
    let value = text
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .ok_or_else(|| {
            format!(
                "desktop release draft stage: {} omitted marker {key}",
                path.display()
            )
        })?
        .trim();
    if !is_lower_hex64(value) {
        return Err(format!(
            "desktop release draft stage: {} has invalid {key}",
            path.display()
        ));
    }
    Ok(value.to_owned())
}

fn verify_attestation(
    ctx: &Context,
    subject: &Path,
    bundle: &Path,
    predicate: &str,
) -> Result<(), String> {
    let common = vec![
        "attestation".to_owned(),
        "verify".to_owned(),
        subject.to_string_lossy().into_owned(),
        "-R".to_owned(),
        ctx.repository.clone(),
        "--signer-workflow".to_owned(),
        ctx.signer_workflow.clone(),
        "--source-digest".to_owned(),
        ctx.commit_sha.clone(),
        "--source-ref".to_owned(),
        "refs/heads/main".to_owned(),
        "--predicate-type".to_owned(),
        predicate.to_owned(),
        "--deny-self-hosted-runners".to_owned(),
    ];
    let mut with_bundle = common.clone();
    with_bundle.push("--bundle".to_owned());
    with_bundle.push(bundle.to_string_lossy().into_owned());
    checked_status(None, "gh", &with_bundle)?;
    checked_status(None, "gh", &common)
}

struct Qualified {
    win_installer: PathBuf,
    win_raw: PathBuf,
    win_bundle: PathBuf,
    win_sbom: PathBuf,
    win_sbom_bundle: PathBuf,
    mac_dmg: PathBuf,
    mac_zip: PathBuf,
    mac_bundle: PathBuf,
    mac_sbom: PathBuf,
    mac_sbom_bundle: PathBuf,
}

fn verify_qualified_artifacts(ctx: &Context, win: &Path, mac: &Path) -> Result<Qualified, String> {
    let q = Qualified {
        win_installer: win.join("KaspaGateway-windows-x64-nsis.exe"),
        win_raw: win.join("kaspa-gateway-desktop-windows-x64.exe"),
        win_bundle: win.join("WINDOWS_BUILD_PROVENANCE.sigstore.json"),
        win_sbom: win.join("WINDOWS_SBOM.spdx.json"),
        win_sbom_bundle: win.join("WINDOWS_SBOM_ATTESTATION.sigstore.json"),
        mac_dmg: mac.join("KaspaGateway-macos-universal.dmg"),
        mac_zip: mac.join("KaspaGateway-macos-universal-app.zip"),
        mac_bundle: mac.join("MACOS_BUILD_PROVENANCE.sigstore.json"),
        mac_sbom: mac.join("MACOS_SBOM.spdx.json"),
        mac_sbom_bundle: mac.join("MACOS_SBOM_ATTESTATION.sigstore.json"),
    };
    let win_smoke = win.join("WINDOWS_INSTALLER_SMOKE.txt");
    let mac_smoke = mac.join("MACOS_DMG_SMOKE.txt");
    require_nonempty(&[
        &q.win_installer,
        &q.win_raw,
        &q.win_bundle,
        &q.win_sbom,
        &q.win_sbom_bundle,
        &win_smoke,
        &win.join("SHA256SUMS"),
        &q.mac_dmg,
        &q.mac_zip,
        &q.mac_bundle,
        &q.mac_sbom,
        &q.mac_sbom_bundle,
        &mac_smoke,
        &mac.join("SHA256SUMS"),
    ])?;
    verify_checksum_manifest(win)?;
    verify_checksum_manifest(mac)?;
    require_markers(
        &win_smoke,
        &[
            format!("REQUESTED_COMMIT_SHA={}", ctx.commit_sha),
            "WINDOWS_INSTALLER_SMOKE=PASS".to_owned(),
            "WINDOWS_UNINSTALL_SMOKE=PASS".to_owned(),
            "TESTNET13_LIVE_SMOKE=NOT_RUN_EXPERIMENTAL".to_owned(),
        ],
    )?;
    let _installed_payload_sha256 =
        require_lower_hex64_marker(&win_smoke, "WINDOWS_INSTALLED_EXE_SHA256")?;
    require_markers(
        &mac_smoke,
        &[
            format!("REQUESTED_COMMIT_SHA={}", ctx.commit_sha),
            "MACOS_DMG_SMOKE=PASS".to_owned(),
            "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64".to_owned(),
            "TESTNET13_LIVE_SMOKE=NOT_RUN_EXPERIMENTAL".to_owned(),
        ],
    )?;
    for sbom in [&q.win_sbom, &q.mac_sbom] {
        if read_json(sbom)?.get("spdxVersion").and_then(Value::as_str) != Some("SPDX-2.3") {
            return Err(format!(
                "desktop release draft stage: invalid SPDX version in {}",
                sbom.display()
            ));
        }
    }
    for subject in [&q.win_installer, &q.win_raw] {
        verify_attestation(ctx, subject, &q.win_bundle, SIGNER_PREDICATE)?;
        verify_attestation(ctx, subject, &q.win_sbom_bundle, SBOM_PREDICATE)?;
    }
    for subject in [&q.mac_dmg, &q.mac_zip] {
        verify_attestation(ctx, subject, &q.mac_bundle, SIGNER_PREDICATE)?;
        verify_attestation(ctx, subject, &q.mac_sbom_bundle, SBOM_PREDICATE)?;
    }
    Ok(q)
}

fn copy_named(source: &Path, stage: &Path, name: String) -> Result<PathBuf, String> {
    let destination = stage.join(name);
    fs::copy(source, &destination).map_err(|error| {
        format!(
            "desktop release draft stage: copy {}: {error}",
            source.display()
        )
    })?;
    Ok(destination)
}

fn assemble_assets(ctx: &Context, q: &Qualified, stage: &Path) -> Result<Vec<PathBuf>, String> {
    reset_dir(stage)?;
    let short = &ctx.commit_sha[..7];
    let items = [
        (
            &q.win_installer,
            format!("KASPA_GATEWAY_WINDOWS_X64_NSIS_{}_{short}.exe", ctx.version),
        ),
        (
            &q.win_raw,
            format!("KASPA_GATEWAY_WINDOWS_X64_RAW_{}_{short}.exe", ctx.version),
        ),
        (
            &q.mac_dmg,
            format!(
                "KASPA_GATEWAY_MACOS_UNIVERSAL_DMG_{}_{short}.dmg",
                ctx.version
            ),
        ),
        (
            &q.mac_zip,
            format!(
                "KASPA_GATEWAY_MACOS_UNIVERSAL_APP_{}_{short}.zip",
                ctx.version
            ),
        ),
        (
            &q.win_bundle,
            format!(
                "KASPA_GATEWAY_WINDOWS_BUILD_PROVENANCE_{}_{short}.sigstore.json",
                ctx.version
            ),
        ),
        (
            &q.win_sbom,
            format!(
                "KASPA_GATEWAY_WINDOWS_SBOM_{}_{short}.spdx.json",
                ctx.version
            ),
        ),
        (
            &q.win_sbom_bundle,
            format!(
                "KASPA_GATEWAY_WINDOWS_SBOM_ATTESTATION_{}_{short}.sigstore.json",
                ctx.version
            ),
        ),
        (
            &q.mac_bundle,
            format!(
                "KASPA_GATEWAY_MACOS_BUILD_PROVENANCE_{}_{short}.sigstore.json",
                ctx.version
            ),
        ),
        (
            &q.mac_sbom,
            format!("KASPA_GATEWAY_MACOS_SBOM_{}_{short}.spdx.json", ctx.version),
        ),
        (
            &q.mac_sbom_bundle,
            format!(
                "KASPA_GATEWAY_MACOS_SBOM_ATTESTATION_{}_{short}.sigstore.json",
                ctx.version
            ),
        ),
    ];
    let mut staged = Vec::new();
    for (source, name) in items {
        staged.push(copy_named(source, stage, name)?);
    }
    staged.sort();
    let sums = staged
        .iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            Ok(format!("{}  {name}\n", sha256(path)?))
        })
        .collect::<Result<String, String>>()?;
    let sum_path = stage.join("SHA256SUMS.txt");
    fs::write(&sum_path, sums)
        .map_err(|error| format!("desktop release draft stage: write SHA256SUMS.txt: {error}"))?;
    staged.push(sum_path);
    Ok(staged)
}

fn create_draft(ctx: &Context, assets: &[PathBuf]) -> Result<(), String> {
    let tag = format!("desktop-v{}", ctx.version);
    let notes = format!(
        "Build provenance and SPDX 2.3 SBOM attestations are attached.\n\nSource commit: {}\nQualified Desktop Artifacts run: {}\n\nThis workflow creates a draft only. Publication requires a separate immutable-release gate.",
        ctx.commit_sha, ctx.artifact_run_id
    );
    let mut args = vec!["release".to_owned(), "create".to_owned(), tag];
    args.extend(
        assets
            .iter()
            .map(|path| path.to_string_lossy().into_owned()),
    );
    args.extend([
        "-R".to_owned(),
        ctx.repository.clone(),
        "--draft".to_owned(),
        "--target".to_owned(),
        ctx.commit_sha.clone(),
        "--title".to_owned(),
        format!("Kaspa Gateway Desktop {}", ctx.version),
        "--generate-notes".to_owned(),
        "--notes".to_owned(),
        notes,
        "--fail-on-no-commits".to_owned(),
    ]);
    checked_status(None, "gh", &args)
}

fn resolve_created_draft(ctx: &Context) -> Result<u64, String> {
    let tag = format!("desktop-v{}", ctx.version);
    for _ in 0..5 {
        let pages = release_pages(&ctx.repository)?;
        let matches = matching_draft(&pages, &tag, &ctx.commit_sha);
        if matches.len() == 1 {
            return matches[0]
                .get("id")
                .and_then(Value::as_u64)
                .ok_or_else(|| "desktop release draft stage: draft release ID invalid".to_owned());
        }
        if matches.len() > 1 {
            return Err("desktop release draft stage: multiple matching drafts found".to_owned());
        }
        thread::sleep(Duration::from_secs(2));
    }
    Err("desktop release draft stage: created draft could not be resolved uniquely".to_owned())
}

fn verify_draft(ctx: &Context, release_id: u64, stage: &Path) -> Result<(), String> {
    let release = gh_json(&[
        "api".to_owned(),
        format!("repos/{}/releases/{release_id}", ctx.repository),
    ])?;
    let tag = format!("desktop-v{}", ctx.version);
    if release.get("id").and_then(Value::as_u64) != Some(release_id)
        || release.get("draft").and_then(Value::as_bool) != Some(true)
        || !release.get("published_at").is_some_and(Value::is_null)
        || release.get("prerelease").and_then(Value::as_bool) != Some(false)
        || release.get("tag_name").and_then(Value::as_str) != Some(tag.as_str())
        || release.get("target_commitish").and_then(Value::as_str) != Some(ctx.commit_sha.as_str())
    {
        return Err(
            "desktop release draft stage: created release object failed identity checks".to_owned(),
        );
    }
    let mut expected = fs::read_dir(stage)
        .map_err(|error| format!("desktop release draft stage: list staged assets: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("desktop release draft stage: staged asset entry: {error}"))?;
    expected.sort_by_key(|entry| entry.file_name());
    let remote = release
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| "desktop release draft stage: release omitted assets".to_owned())?;
    if remote.len() != expected.len() {
        return Err("desktop release draft stage: release asset count mismatch".to_owned());
    }
    for entry in expected {
        let name = entry.file_name().to_string_lossy().into_owned();
        let asset = remote
            .iter()
            .find(|asset| asset.get("name").and_then(Value::as_str) == Some(name.as_str()))
            .ok_or_else(|| format!("desktop release draft stage: missing remote asset {name}"))?;
        if asset.get("size").and_then(Value::as_u64).unwrap_or(0) == 0 {
            return Err(format!(
                "desktop release draft stage: zero-sized remote asset {name}"
            ));
        }
        let digest = format!("sha256:{}", sha256(&entry.path())?);
        if asset.get("digest").and_then(Value::as_str) != Some(digest.as_str()) {
            return Err(format!(
                "desktop release draft stage: digest mismatch for remote asset {name}"
            ));
        }
    }
    if let Some(output) = &ctx.github_output {
        fs::write(output, format!("release_id={release_id}\n")).map_err(|error| {
            format!("desktop release draft stage: write GITHUB_OUTPUT: {error}")
        })?;
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    let ctx = Context::from_env()?;
    validate_request(&ctx)?;
    validate_remote_preconditions(&ctx)?;
    validate_checkout_and_version(root, &ctx)?;
    validate_artifact_run(&ctx)?;
    let qualified_root = ctx.runner_temp.join("qualified-artifacts");
    let stage = ctx.runner_temp.join("release-assets");
    let (win, mac) = download_artifacts(&ctx, &qualified_root)?;
    let qualified = verify_qualified_artifacts(&ctx, &win, &mac)?;
    let assets = assemble_assets(&ctx, &qualified, &stage)?;
    create_draft(&ctx, &assets)?;
    let release_id = resolve_created_draft(&ctx)?;
    verify_draft(&ctx, release_id, &stage)?;
    Ok(format!(
        "DRAFT_RELEASE_ID={release_id}\nDRAFT_RELEASE=desktop-v{}\nDRAFT_ONLY=YES\nPUBLISHED=NO\nNEXT_ACTION=ADMIN_IMMUTABILITY_GATE_THEN_PUBLISH",
        ctx.version
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_stable_inputs() {
        assert!(is_version("1.2.3"));
        assert!(!is_version("1.2"));
        assert!(!is_version("1.2.3-beta"));
        assert!(is_lower_hex40("0123456789abcdef0123456789abcdef01234567"));
        assert!(!is_lower_hex40("0123456789ABCDEF0123456789abcdef01234567"));
        assert!(is_lower_hex64(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(!is_lower_hex64(
            "0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(is_digits("123456"));
        assert!(!is_digits("12a"));
    }

    #[test]
    fn parses_package_version_only() {
        let text =
            "[workspace]\nversion = \"9.9.9\"\n[package]\nname = \"x\"\nversion = \"1.2.3\"\n";
        assert_eq!(cargo_package_version(text).as_deref(), Some("1.2.3"));
    }

    #[test]
    fn matching_draft_requires_exact_identity() {
        let pages = serde_json::json!([[{
            "id": 7,
            "draft": true,
            "tag_name": "desktop-v1.2.3",
            "target_commitish": "abc"
        }]]);
        assert_eq!(matching_draft(&pages, "desktop-v1.2.3", "abc").len(), 1);
        assert!(matching_draft(&pages, "desktop-v1.2.4", "abc").is_empty());
    }
}
