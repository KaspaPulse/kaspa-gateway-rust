#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
KSSS = ROOT / ".security" / "ksss"

class TrustAcceptanceError(RuntimeError):
    pass

def sha256_file(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise TrustAcceptanceError(f"UNSAFE_OR_MISSING_FILE:{path}")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def canonical_sha256(value: Any) -> str:
    payload = json.dumps(
        value, ensure_ascii=False, sort_keys=True,
        separators=(",", ":"), allow_nan=False,
    ).encode()
    return hashlib.sha256(payload).hexdigest()

def load_object(path: Path, label: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise TrustAcceptanceError(f"{label}_MISSING_OR_UNSAFE")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise TrustAcceptanceError(f"{label}_INVALID_JSON") from exc
    if not isinstance(value, dict):
        raise TrustAcceptanceError(f"{label}_INVALID")
    return value
def parse_time(value: str, label: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except Exception as exc:
        raise TrustAcceptanceError(f"{label}_INVALID") from exc
    if parsed.tzinfo is None:
        raise TrustAcceptanceError(f"{label}_TIMEZONE_REQUIRED")
    return parsed

def evidence_dir(repo: Path, policy: dict[str, Any]) -> Path:
    release = str(policy.get("ksss_release", ""))
    path = repo / ".security/ksss/trust/evidence" / release
    if not path.is_dir():
        raise TrustAcceptanceError("EVIDENCE_DIRECTORY_MISSING")
    return path

def parse_sha256sums(path: Path) -> dict[str, str]:
    entries: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        parts = line.split(None, 1)
        if len(parts) != 2 or len(parts[0]) != 64:
            raise TrustAcceptanceError("SHA256SUMS_LINE_INVALID")
        name = parts[1].lstrip("*")
        if "/" in name or "\\" in name or name in entries:
            raise TrustAcceptanceError("SHA256SUMS_PATH_INVALID")
        entries[name] = parts[0]
    return entries

def verify_manifest(directory: Path) -> dict[str, str]:
    entries = parse_sha256sums(directory / "SHA256SUMS")
    actual = {p.name for p in directory.iterdir() if p.is_file() and p.name != "SHA256SUMS"}
    if actual != set(entries):
        raise TrustAcceptanceError("EVIDENCE_FILE_SET_MISMATCH")
    for name, expected in entries.items():
        if sha256_file(directory / name) != expected:
            raise TrustAcceptanceError(f"EVIDENCE_DIGEST_MISMATCH:{name}")
    return entries
def verify_structure(repo: Path, explicit_time: str) -> dict[str, Any]:
    policy = load_object(repo / ".security/ksss/trust-policy.json", "TRUST_POLICY")
    directory = evidence_dir(repo, policy)
    verify_manifest(directory)
    root = load_object(directory / "ksss-trust-root-v1.json", "TRUST_ROOT")
    if sha256_file(directory / "ksss-trust-root-v1.json") != policy["trust_root_file_sha256"]:
        raise TrustAcceptanceError("TRUST_ROOT_FILE_DIGEST_MISMATCH")
    if canonical_sha256(root) != policy["trust_root_canonical_sha256"]:
        raise TrustAcceptanceError("TRUST_ROOT_CANONICAL_DIGEST_MISMATCH")
    now = parse_time(explicit_time, "EXPLICIT_TIME")
    if now < parse_time(root["valid_from"], "TRUST_ROOT_VALID_FROM"):
        raise TrustAcceptanceError("TRUST_ROOT_NOT_YET_VALID")
    if now >= parse_time(root["expires_at"], "TRUST_ROOT_EXPIRES_AT"):
        raise TrustAcceptanceError("TRUST_ROOT_EXPIRED")
    for key in ("trusted_issuer", "trusted_signer_or_workflow_identity"):
        if root[key] != policy[key]:
            raise TrustAcceptanceError(f"TRUST_ROOT_{key.upper()}_MISMATCH")
    if root["minimum_allowed_ksss_sequence"] > policy["expected_sequence"]:
        raise TrustAcceptanceError("KSSS_SEQUENCE_BELOW_ROOT_FLOOR")

    release = policy["ksss_release"]
    identity = load_object(directory / f"ksss-release-identity-{release}.json", "RELEASE_IDENTITY")
    metadata = load_object(directory / f"ksss-trusted-release-{release}.json", "RELEASE_METADATA")
    release_statement = load_object(directory / f"ksss-release-attestation-{release}.intoto.json", "RELEASE_STATEMENT")
    runtime_statement = load_object(directory / f"ksss-consumer-runtime-{release}.intoto.json", "RUNTIME_STATEMENT")
    expected_release = policy["ksss_release"]
    expected_source = policy["ksss_source_sha"]
    expected_bundle = policy["policy_bundle_digest"]
    if identity["KSSS_RELEASE"] != expected_release or identity["SOURCE_SHA"] != expected_source:
        raise TrustAcceptanceError("RELEASE_IDENTITY_MISMATCH")
    if identity["POLICY_BUNDLE_DIGEST"] != expected_bundle:
        raise TrustAcceptanceError("POLICY_BUNDLE_DIGEST_MISMATCH")
    if metadata["release"] != expected_release or metadata["source_sha"] != expected_source:
        raise TrustAcceptanceError("TRUSTED_RELEASE_IDENTITY_MISMATCH")
    if metadata["sequence"] != policy["expected_sequence"]:
        raise TrustAcceptanceError("TRUSTED_RELEASE_SEQUENCE_MISMATCH")
    if metadata["policy_bundle_digest"] != expected_bundle:
        raise TrustAcceptanceError("TRUSTED_RELEASE_BUNDLE_MISMATCH")
    if release_statement["subject"][0]["digest"]["sha256"] != expected_bundle:
        raise TrustAcceptanceError("RELEASE_STATEMENT_SUBJECT_MISMATCH")
    if release_statement["predicate"]["sourceSha"] != expected_source:
        raise TrustAcceptanceError("RELEASE_STATEMENT_SOURCE_MISMATCH")

    artifact = directory / policy["runtime_artifact_filename"]
    runtime_sha = sha256_file(artifact)
    if runtime_sha != policy["runtime_artifact_sha256"]:
        raise TrustAcceptanceError("RUNTIME_ARTIFACT_SHA256_MISMATCH")
    predicate = runtime_statement["predicate"]
    subject = runtime_statement["subject"][0]
    if subject["digest"]["sha256"] != runtime_sha:
        raise TrustAcceptanceError("RUNTIME_STATEMENT_SUBJECT_MISMATCH")
    if predicate["ksssRelease"] != expected_release or predicate["sourceSha"] != expected_source:
        raise TrustAcceptanceError("RUNTIME_STATEMENT_IDENTITY_MISMATCH")
    if predicate["runtimeSequence"] != policy["runtime_sequence"]:
        raise TrustAcceptanceError("RUNTIME_SEQUENCE_MISMATCH")
    if predicate["runtimeFilesDigest"] != policy["runtime_files_digest"]:
        raise TrustAcceptanceError("RUNTIME_FILES_DIGEST_MISMATCH")
    if predicate["networkRequiredForExecution"] is not False:
        raise TrustAcceptanceError("RUNTIME_NETWORK_CONTRACT_INVALID")
    if predicate["externalKsssCheckoutRequired"] is not False:
        raise TrustAcceptanceError("RUNTIME_EXTERNAL_CHECKOUT_CONTRACT_INVALID")
    return {"policy": policy, "directory": directory, "root": root}
def cosign_verify(
    *, cosign: str, statement: Path, bundle: Path,
    trusted_root: Path, identity: str, issuer: str,
) -> None:
    executable = shutil.which(cosign) if "/" not in cosign else cosign
    if not executable or not Path(executable).is_file():
        raise TrustAcceptanceError("COSIGN_NOT_AVAILABLE")
    completed = subprocess.run(
        [
            executable, "verify-blob", str(statement),
            "--bundle", str(bundle),
            "--certificate-identity", identity,
            "--certificate-oidc-issuer", issuer,
            "--trusted-root", str(trusted_root),
        ],
        text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if completed.returncode:
        raise TrustAcceptanceError("COSIGN_VERIFY_FAILED:" + completed.stderr.strip()[-400:])

def verify(
    *, repo: Path, explicit_time: str,
    require_cryptographic: bool, cosign: str,
) -> dict[str, Any]:
    structural = verify_structure(repo, explicit_time)
    policy = structural["policy"]
    directory = structural["directory"]
    release = policy["ksss_release"]
    crypto = "NOT_REQUESTED"
    if require_cryptographic:
        trusted_root = directory / policy["sigstore_trusted_root_filename"]
        if sha256_file(trusted_root) != policy["sigstore_trusted_root_sha256"]:
            raise TrustAcceptanceError("SIGSTORE_TRUSTED_ROOT_DIGEST_MISMATCH")
        identity = policy["trusted_signer_or_workflow_identity"]
        issuer = policy["trusted_issuer"]
        cosign_verify(
            cosign=cosign,
            statement=directory / f"ksss-release-attestation-{release}.intoto.json",
            bundle=directory / f"ksss-release-attestation-{release}.sigstore.json",
            trusted_root=trusted_root, identity=identity, issuer=issuer,
        )
        cosign_verify(
            cosign=cosign,
            statement=directory / f"ksss-consumer-runtime-{release}.intoto.json",
            bundle=directory / f"ksss-consumer-runtime-{release}.sigstore.json",
            trusted_root=trusted_root, identity=identity, issuer=issuer,
        )
        crypto = "PASS"
    return {
        "status": "PASS",
        "repository": "KaspaPulse/kaspa-gateway-rust",
        "ksssRelease": policy["ksss_release"],
        "ksssSourceSha": policy["ksss_source_sha"],
        "policyBundleDigest": policy["policy_bundle_digest"],
        "runtimeArtifactSha256": policy["runtime_artifact_sha256"],
        "runtimeFilesDigest": policy["runtime_files_digest"],
        "runtimeSequence": policy["runtime_sequence"],
        "trustRoot": policy["trust_root_version"],
        "structuralVerification": "PASS",
        "cryptographicVerification": crypto,
        "runtimeCryptographicVerification": crypto,
        "policyEnforcementAuthorized": require_cryptographic and crypto == "PASS",
        "PRODUCTION_CHANGE": False,
        "DEPLOYMENT": False,
        "CLOUDFLARE_CHANGE": False,
        "D1_MUTATION": False,
        "USER_DATA_ACCESS": False,
    }

def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    sub = value.add_subparsers(dest="action", required=True)
    check = sub.add_parser("verify")
    check.add_argument("--repo", type=Path, default=ROOT)
    check.add_argument("--explicit-time", required=True)
    check.add_argument("--require-cryptographic", action="store_true")
    check.add_argument("--cosign", default="cosign")
    check.add_argument("--output", type=Path)
    return value

def main() -> int:
    args = parser().parse_args()
    try:
        result = verify(
            repo=args.repo.resolve(),
            explicit_time=args.explicit_time,
            require_cryptographic=args.require_cryptographic,
            cosign=args.cosign,
        )
    except (OSError, ValueError, TrustAcceptanceError, subprocess.SubprocessError) as exc:
        print(json.dumps({"status": "FAIL", "reason": str(exc)}, sort_keys=True))
        return 1
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
