#!/usr/bin/env python3
from __future__ import annotations

import contextlib
import hashlib
import importlib
import json
import os
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any, Iterator

ROOT = Path(__file__).resolve().parents[2]
KSSS = ROOT / ".security" / "ksss"
ARCHIVE_ROOT = "ksss-consumer-runtime-v1"
RUNTIME_MODULES = (
    "bundle_verify", "canonical", "classify_changes", "consumer_api",
    "knowledge", "resolver", "runtime_verify", "schema_registry",
)

class RuntimeLoaderError(RuntimeError):
    pass

def load_json(path: Path, label: str) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise RuntimeLoaderError(f"{label}_MISSING_OR_UNSAFE")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise RuntimeLoaderError(f"{label}_INVALID_JSON") from exc
    if not isinstance(value, dict):
        raise RuntimeLoaderError(f"{label}_INVALID")
    return value
def sha256_file(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise RuntimeLoaderError(f"RUNTIME_FILE_MISSING_OR_UNSAFE:{path}")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()

def load_policy(repo: Path = ROOT) -> dict[str, Any]:
    return load_json(repo / ".security/ksss/trust-policy.json", "TRUST_POLICY")

def evidence_dir(repo: Path, policy: dict[str, Any]) -> Path:
    release = str(policy.get("ksss_release", ""))
    if not release or "/" in release or "\\" in release:
        raise RuntimeLoaderError("TRUST_POLICY_RELEASE_INVALID")
    path = repo / ".security/ksss/trust/evidence" / release
    if not path.is_dir():
        raise RuntimeLoaderError("RUNTIME_EVIDENCE_DIRECTORY_MISSING")
    return path

def runtime_artifact_path(repo: Path, policy: dict[str, Any]) -> Path:
    name = str(policy.get("runtime_artifact_filename", ""))
    if not name or "/" in name or "\\" in name:
        raise RuntimeLoaderError("RUNTIME_ARTIFACT_FILENAME_INVALID")
    return evidence_dir(repo, policy) / name

def verify_artifact_pin(repo: Path = ROOT) -> tuple[dict[str, Any], Path]:
    policy = load_policy(repo)
    if policy["runtime_sequence"] < policy["minimum_allowed_runtime_sequence"]:
        raise RuntimeLoaderError("RUNTIME_SEQUENCE_BELOW_FLOOR")
    artifact = runtime_artifact_path(repo, policy)
    expected = str(policy.get("runtime_artifact_sha256", ""))
    if len(expected) != 64 or sha256_file(artifact) != expected:
        raise RuntimeLoaderError("RUNTIME_ARTIFACT_SHA256_MISMATCH")
    return policy, artifact
def _relative_member(name: str) -> PurePosixPath:
    prefix = ARCHIVE_ROOT + "/"
    if not name.startswith(prefix):
        raise RuntimeLoaderError("RUNTIME_ARCHIVE_ROOT_INVALID")
    relative = name[len(prefix):]
    pure = PurePosixPath(relative)
    if (
        not relative or pure.is_absolute() or ".." in pure.parts
        or relative.startswith("./") or "\\" in relative
    ):
        raise RuntimeLoaderError("RUNTIME_ARCHIVE_PATH_INVALID")
    return pure

def extract_verified_runtime(
    destination: Path, *, repo: Path = ROOT
) -> tuple[Path, dict[str, Any]]:
    policy, artifact = verify_artifact_pin(repo)
    if destination.exists() and any(destination.iterdir()):
        raise RuntimeLoaderError("RUNTIME_DESTINATION_NOT_EMPTY")
    destination.mkdir(parents=True, exist_ok=True)
    try:
        archive = tarfile.open(artifact, mode="r:gz")
    except tarfile.TarError as exc:
        raise RuntimeLoaderError("RUNTIME_ARCHIVE_INVALID") from exc
    seen: set[str] = set()
    with archive:
        for member in archive.getmembers():
            if not member.isfile():
                raise RuntimeLoaderError("RUNTIME_ARCHIVE_NON_REGULAR_MEMBER")
            relative = _relative_member(member.name)
            key = relative.as_posix()
            if key in seen:
                raise RuntimeLoaderError("RUNTIME_ARCHIVE_DUPLICATE_MEMBER")
            seen.add(key)
            source = archive.extractfile(member)
            if source is None:
                raise RuntimeLoaderError("RUNTIME_ARCHIVE_MEMBER_UNREADABLE")
            target = destination / ARCHIVE_ROOT / Path(*relative.parts)
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as handle:
                while block := source.read(1024 * 1024):
                    handle.write(block)
            os.chmod(target, 0o644)
    runtime_root = destination / ARCHIVE_ROOT
    verifier = runtime_root / "scripts/runtime_verify.py"
    if not verifier.is_file():
        raise RuntimeLoaderError("RUNTIME_SELF_VERIFIER_MISSING")
    completed = subprocess.run(
        [sys.executable, "-I", str(verifier), "--root", str(runtime_root)],
        text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        env={**{key: os.environ[key] for key in ("SYSTEMROOT", "WINDIR", "TEMP", "TMP") if key in os.environ}, "PYTHONDONTWRITEBYTECODE": "1", "PYTHONNOUSERSITE": "1"},
        timeout=30, check=False,
    )
    if completed.returncode:
        raise RuntimeLoaderError("RUNTIME_SELF_VERIFY_FAILED:" + completed.stderr.strip()[-400:])
    try:
        verification = json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeLoaderError("RUNTIME_SELF_VERIFY_OUTPUT_INVALID") from exc
    if verification.get("status") != "PASS":
        raise RuntimeLoaderError("RUNTIME_SELF_VERIFY_NOT_PASS")
    manifest = load_json(runtime_root / "runtime-manifest.json", "RUNTIME_MANIFEST")
    expected_pairs = (
        ("ksss_release", "ksss_release"),
        ("ksss_source_sha", "ksss_source_sha"),
        ("runtime_sequence", "runtime_sequence"),
        ("runtime_files_digest", "runtime_files_digest"),
        ("policy_bundle_digest", "policy_bundle_digest"),
    )
    for manifest_key, policy_key in expected_pairs:
        if manifest.get(manifest_key) != policy.get(policy_key):
            raise RuntimeLoaderError(f"RUNTIME_POLICY_BINDING_MISMATCH:{manifest_key}")
    if manifest.get("network_required_for_execution") is not False:
        raise RuntimeLoaderError("RUNTIME_NETWORK_REQUIREMENT_INVALID")
    if manifest.get("external_ksss_checkout_required") is not False:
        raise RuntimeLoaderError("RUNTIME_EXTERNAL_CHECKOUT_INVALID")
    return runtime_root, manifest
@contextlib.contextmanager
def runtime_context(repo: Path = ROOT) -> Iterator[tuple[Any, Path, dict[str, Any]]]:
    collisions = [name for name in RUNTIME_MODULES if name in sys.modules]
    if collisions:
        raise RuntimeLoaderError("RUNTIME_MODULE_NAME_COLLISION:" + ",".join(sorted(collisions)))
    old_no_bytecode = sys.dont_write_bytecode
    with tempfile.TemporaryDirectory(prefix="kgw-ksss-runtime-") as directory:
        runtime_root, manifest = extract_verified_runtime(Path(directory), repo=repo)
        scripts = runtime_root / "scripts"
        sys.path.insert(0, str(scripts))
        sys.dont_write_bytecode = True
        try:
            api = importlib.import_module("consumer_api")
            if getattr(api, "RUNTIME_API_VERSION", None) != "1.0.0":
                raise RuntimeLoaderError("RUNTIME_API_VERSION_MISMATCH")
            yield api, runtime_root, manifest
        finally:
            sys.dont_write_bytecode = old_no_bytecode
            try:
                sys.path.remove(str(scripts))
            except ValueError:
                pass
            prefix = str(runtime_root)
            for name in RUNTIME_MODULES:
                module = sys.modules.get(name)
                module_file = getattr(module, "__file__", "") if module else ""
                if module_file and str(module_file).startswith(prefix):
                    sys.modules.pop(name, None)

def offline_probe(repo: Path = ROOT) -> dict[str, Any]:
    with runtime_context(repo) as (api, _, manifest):
        classifications = {
            "code": api.classify_paths(["src/App.jsx"]),
            "dependency": api.classify_paths(["package-lock.json"]),
            "workflow": api.classify_paths([".github/workflows/quality.yml"]),
            "policy": api.classify_paths([".security/ksss/repository-policy.yaml"]),
        }
        expected = {
            "code": ["CODE_ONLY_CHANGE"],
            "dependency": ["DEPENDENCY_CHANGE"],
            "workflow": ["WORKFLOW_CHANGE"],
            "policy": ["POLICY_CHANGE"],
        }
        if classifications != expected:
            raise RuntimeLoaderError("RUNTIME_CLASSIFIER_PROBE_MISMATCH")
        return {
            "status": "PASS",
            "ksss_release": manifest["ksss_release"],
            "ksss_source_sha": manifest["ksss_source_sha"],
            "runtime_sequence": manifest["runtime_sequence"],
            "runtime_files_digest": manifest["runtime_files_digest"],
            "network_required_for_execution": False,
            "external_ksss_checkout_required": False,
            "classifications": classifications,
        }

if __name__ == "__main__":
    try:
        print(json.dumps(offline_probe(), sort_keys=True))
    except (OSError, ValueError, RuntimeLoaderError, subprocess.SubprocessError) as exc:
        print(json.dumps({"status": "FAIL", "reason": str(exc)}, sort_keys=True))
        raise SystemExit(1)
