#!/usr/bin/env python3
"""KGW adapter for the signed KSSS Consumer Runtime; no duplicate policy engine."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

import runtime_loader
import trust_acceptance

ROOT = Path(__file__).resolve().parents[2]
KSSS = ROOT / ".security/ksss"
REPOSITORY = "KaspaPulse/kaspa-gateway-rust"


class ConsumerError(ValueError):
    pass


def load(relative):
    return runtime_loader.load_json(KSSS / relative, relative)


def digest(path):
    return runtime_loader.sha256_file(Path(path))


def git(*args):
    return subprocess.check_output(["git", "-C", str(ROOT), *args])


def identity():
    paths = sorted(set(filter(None, git("ls-files", "-z").decode().split("\0"))))
    untracked = sorted(filter(None, git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0")))
    def fingerprint(names):
        value = hashlib.sha256()
        for name in names:
            path = ROOT / name
            value.update(name.replace("\\", "/").encode() + b"\0")
            if path.is_symlink():
                value.update(os.readlink(path).encode())
            elif path.is_file():
                value.update(hashlib.sha256(path.read_bytes()).digest())
            else:
                value.update(b"<missing-or-non-file>")
        return value.hexdigest()
    return {
        "source_sha": git("rev-parse", "HEAD").decode().strip(),
        "git_tree": git("rev-parse", "HEAD^{tree}").decode().strip(),
        "tree_hash": fingerprint(paths),
        "worktree_state": {
            "staged_fingerprint": hashlib.sha256(git("diff", "--cached", "--binary")).hexdigest(),
            "unstaged_fingerprint": hashlib.sha256(git("diff", "--binary")).hexdigest(),
            "untracked_fingerprint": fingerprint(untracked),
            "submodule_fingerprint": hashlib.sha256(git("submodule", "status", "--recursive")).hexdigest(),
        },
    }


def classify(api, paths):
    # Classify each path through the official API. Union preserves mixed code +
    # dependency/policy triggers that a single aggregate classification can omit.
    return sorted({kind for path in paths for kind in api.classify_paths([path])})


def resolve(api, explicit_time):
    policy, applicability = load("repository-policy.yaml"), load("applicability-v1.json")
    strengthening = load("local-strengthening.json")
    api.assert_valid_document(policy)
    api.assert_valid_document(strengthening)
    if policy["repository"] != REPOSITORY or applicability["repository"] != REPOSITORY:
        raise ConsumerError("REPOSITORY_IDENTITY_MISMATCH")
    catalog = {item["id"]: item for item in api.control_catalog()["controls"]}
    required = set(applicability["required_controls"])
    if not required or required - catalog.keys():
        raise ConsumerError("APPLICABILITY_UNKNOWN_OR_EMPTY")
    risk_required = set(policy["controls"]["additional_required"])
    if not required.issubset(risk_required):
        raise ConsumerError("RISK_FLOOR_WEAKENED")
    base = {"repository": REPOSITORY, "controls": {
        key: {"required": False, "exception_allowed": item["exception_allowed"], "parameters": {}}
        for key, item in sorted(catalog.items())
    }}
    effective = api.resolve_effective_policy(
        base_policy=base,
        risk_classification={"classification": policy["risk"], "required_controls": sorted(risk_required)},
        applicability={"required_controls": sorted(required)},
        repository_profile={"profile": policy["audit_profile"], "required_controls":
                            sorted(required.intersection(api.profile(policy["audit_profile"])["required_controls"]))},
        local_strengthening=strengthening, exceptions=[],
        explicit_time=explicit_time, resolver_version="1.0.0",
    )
    if set(effective["required_controls"]) != required:
        raise ConsumerError("REQUIRED_CONTROL_RESOLUTION_DRIFT")
    return effective


def adoption(api, adopted_at):
    trust, policy = load("trust-policy.json"), load("repository-policy.yaml")
    value = {
        "schema_family": "ksss-adoption", "schema_version": "1.0.0",
        "repository": REPOSITORY, "ksss_release": trust["ksss_release"],
        "ksss_source_sha": trust["ksss_source_sha"], "policy_bundle_digest": trust["policy_bundle_digest"],
        "repository_profile": policy["audit_profile"],
        "risk_classification_digest": api.canonical_sha256(policy["risk"]),
        "applicability_digest": api.canonical_sha256(load("applicability-v1.json")),
        "adopted_at": adopted_at,
    }
    api.assert_valid_document(value)
    return value


def learning(api):
    index = load("learning/index.json")
    records = []
    for family, field in (("known-failure", "known_failures"), ("known-good-path", "known_good_paths")):
        for relative in index[field]:
            record = load(relative)
            if api.assert_valid_document(record)[0] != family:
                raise ConsumerError("LEARNING_FAMILY_MISMATCH")
            for ref in record["evidence_refs"]:
                if ref.startswith("repo:") and not (ROOT / ref[5:]).is_file():
                    raise ConsumerError("LEARNING_EVIDENCE_MISSING:" + ref)
            if not record["invalidated_by"] or api.record_reusable(
                record, context=record["applies_when"], active_change_types=record["invalidated_by"][:1]
            ):
                raise ConsumerError("STALE_LEARNING_RECORD_ACCEPTED")
            records.append(record)
    if not index["known_failures"] or not index["known_good_paths"]:
        raise ConsumerError("LEARNING_INVENTORY_EMPTY")
    api.validate_diagnosis_sequence(["PREVIOUS_KNOWLEDGE_SEARCH=PASS", "DIAGNOSIS"])
    return {"known_failures": len(index["known_failures"]), "known_good_paths": len(index["known_good_paths"]),
            "historical_tests_reexecuted": False, "record_validation": "PASS"}


def knowledge_lookup(context_path):
    query = runtime_loader.load_json(context_path, "KNOWLEDGE_QUERY")
    index = load("learning/index.json")
    with runtime_loader.runtime_context(ROOT) as (api, runtime_root, _):
        failures = [load(name) for name in index["known_failures"]]
        good = [load(name) for name in index["known_good_paths"]]
        for family, target in (("global-known-failures", failures), ("global-known-good-paths", good)):
            for path in sorted((runtime_root / "learning" / family).glob("*.json")):
                target.append(runtime_loader.load_json(path, "GLOBAL_KNOWLEDGE"))
        for record in failures + good:
            api.assert_valid_document(record)
        context, changes = query["context"], query.get("active_change_types", [])
        matched_failures = api.lookup_known_failures(
            failures, fingerprint=query["failure_fingerprint"], context=context, active_change_types=changes
        ) if query.get("failure_fingerprint") else []
        matched_good = api.lookup_known_good_paths(
            good, operation_class=query["operation_class"], context=context, active_change_types=changes
        ) if query.get("operation_class") else []
        return {"PREVIOUS_KNOWLEDGE_SEARCH": "PASS",
                "match_status": "MATCH" if matched_failures or matched_good else "NO_VALID_MATCH",
                "known_failures": [{"record_id": r["record_id"], "recommended_action": r.get("recommended_action"),
                                   "evidence_refs": r["evidence_refs"]} for r in matched_failures],
                "known_good_paths": [r["record_id"] for r in matched_good],
                "historical_tests_reexecuted": False}


def reference_parity(manifest):
    # Reuse Gov Forms' exact-source comparison only for the identical signed bytes.
    proof, trust = load("reference-parity.json"), load("trust-policy.json")
    for key in ("ksss_release", "ksss_source_sha", "runtime_artifact_sha256"):
        if proof.get(key) != trust[key]:
            raise ConsumerError("REFERENCE_PARITY_IDENTITY_MISMATCH")
    rows = proof.get("files", [])
    if (proof.get("status") != "PASS" or proof.get("mismatches")
            or proof.get("runtime_files_digest") != manifest["runtime_files_digest"]
            or proof.get("compared_file_count") != 34 or proof.get("byte_equal_count") != 34
            or len(rows) != 34 or len({row["path"] for row in rows}) != 34):
        raise ConsumerError("REFERENCE_PARITY_INCOMPLETE")
    for row in rows:
        expected = manifest["files"].get(row["path"])
        if (not expected or row.get("byte_equal") is not True
                or any(row.get(key) != expected
                       for key in ("manifest_sha256", "runtime_sha256", "source_sha256"))):
            raise ConsumerError("REFERENCE_PARITY_FILE_MISMATCH")
    return {"status": "PASS", "compared_files": 34, "source_comparison_reused": True}


def check(explicit_time):
    trust_acceptance.verify_structure(ROOT, explicit_time)
    with runtime_loader.runtime_context(ROOT) as (api, _, manifest):
        threat = load("threat-model.json")
        errors = api.schema_registry.validate_json_schema(
            threat, api.load_document("schemas/threat-model.schema.json"))
        if errors or threat.get("repository") != REPOSITORY:
            raise ConsumerError("THREAT_MODEL_INVALID:" + ";".join(errors))
        parity = reference_parity(manifest)
        saved = load("ksss-adoption.json")
        if saved != adoption(api, saved["adopted_at"]):
            raise ConsumerError("ADOPTION_SNAPSHOT_DRIFT")
        if load("effective-policy.json") != resolve(api, saved["adopted_at"]):
            raise ConsumerError("EFFECTIVE_POLICY_SNAPSHOT_DRIFT")
        resolve(api, explicit_time)
        states = load("control-state.json")
        required = set(load("applicability-v1.json")["required_controls"])
        if required - states["controls"].keys():
            raise ConsumerError("CONTROL_STATE_INCOMPLETE")
        if states["application_runtime_status"] != "NOT_VERIFIED":
            raise ConsumerError("RUNTIME_CLAIM_REQUIRES_SEPARATE_RECEIPT")
        return {"adoption_status": "PASS", "ksss_release": manifest["ksss_release"],
                "network_required_for_execution": False, "external_ksss_checkout_required": False,
                "reference_parity": parity, "learning": learning(api),
                "application_runtime_status": "NOT_VERIFIED",
                "release_qualification": "NOT_VERIFIED"}


def evaluate(base, explicit_time):
    check(explicit_time)
    if not re.fullmatch(r"[0-9a-f]{40}", base):
        raise ConsumerError("BASE_MUST_BE_EXACT_COMMIT")
    paths = set()
    for args in (("diff", "--name-only", "-z", base, "HEAD", "--"),
                 ("diff", "--name-only", "-z"), ("diff", "--cached", "--name-only", "-z"),
                 ("ls-files", "--others", "--exclude-standard", "-z")):
        paths.update(filter(None, git(*args).decode().split("\0")))
    with runtime_loader.runtime_context(ROOT) as (api, _, _):
        types = classify(api, paths)
        invalidated = sorted(item["id"] for item in api.control_catalog()["controls"]
                             if set(types).intersection(item["freshness"]["invalidated_by"]))
        return {"status": "PASS", "identity": identity(), "change_types": types,
                "invalidated_controls": invalidated, "changed_paths": sorted(paths),
                "automatic_test_execution": False}


def release_check(receipt_path, artifact, evidence_root, expected_environment):
    receipt = runtime_loader.load_json(receipt_path, "RUNTIME_RECEIPT")
    if receipt.get("schema") != "kgw-runtime-qualification-receipt-v1":
        raise ConsumerError("RUNTIME_RECEIPT_SCHEMA_INVALID")
    if receipt.get("environment_fingerprint") != expected_environment:
        raise ConsumerError("RUNTIME_ENVIRONMENT_MISMATCH")
    current = identity()
    contract = load("runtime-contract.json")
    for field in ("source_sha", "tree_hash", "worktree_state"):
        if receipt.get(field) != current[field]:
            raise ConsumerError("RUNTIME_RECEIPT_IDENTITY_MISMATCH:" + field)
    if receipt.get("artifact_sha256") != digest(artifact):
        raise ConsumerError("RUNTIME_ARTIFACT_MISMATCH")
    if not re.fullmatch(r"[0-9a-f]{64}", receipt.get("environment_fingerprint", "")):
        raise ConsumerError("RUNTIME_ENVIRONMENT_IDENTITY_MISSING")
    if receipt.get("host_os") != "windows" or receipt.get("test_kind") != "real-packaged-application":
        raise ConsumerError("REAL_PACKAGED_WINDOWS_EVIDENCE_REQUIRED")
    cases = receipt.get("cases", [])
    if len(cases) != len(contract["cases"]) or {c.get("case_id") for c in cases} != set(contract["cases"]):
        raise ConsumerError("RUNTIME_CASES_INCOMPLETE")
    evidence_root = evidence_root.resolve()
    for case in cases:
        owner = case.get("owner_identity", {})
        if not isinstance(owner.get("pid"), int) or isinstance(owner.get("pid"), bool) or owner["pid"] <= 0:
            raise ConsumerError("EXACT_OWNER_PID_MISSING")
        if not owner.get("start_time") or not owner.get("executable"):
            raise ConsumerError("EXACT_OWNER_IDENTITY_MISSING")
        checks = case.get("checks", {})
        for name in contract["required_checks"]:
            item = checks.get(name, {})
            if item.get("status") != "PASS":
                raise ConsumerError("RUNTIME_CHECK_NOT_PASS:" + case["case_id"] + ":" + name)
            relative = item.get("evidence_path", "")
            candidate = evidence_root / relative
            if not relative or candidate.is_symlink():
                raise ConsumerError("RUNTIME_EVIDENCE_PATH_INVALID")
            path = candidate.resolve()
            if not path.is_relative_to(evidence_root) or not path.is_file():
                raise ConsumerError("RUNTIME_EVIDENCE_PATH_INVALID")
            if item.get("evidence_sha256") != digest(path):
                raise ConsumerError("RUNTIME_EVIDENCE_DIGEST_MISMATCH")
    return {"receipt_validation": "PASS", "application_executed_by_this_command": False,
            "artifact_sha256": receipt["artifact_sha256"], "cases": len(cases)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["check", "materialize", "evaluate", "knowledge", "release-check"])
    parser.add_argument("--explicit-time", default=datetime.now(timezone.utc).isoformat())
    parser.add_argument("--base")
    parser.add_argument("--context", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--artifact", type=Path)
    parser.add_argument("--evidence-root", type=Path)
    parser.add_argument("--environment-fingerprint")
    args = parser.parse_args()
    try:
        if args.action == "materialize":
            trust_acceptance.verify_structure(ROOT, args.explicit_time)
            with runtime_loader.runtime_context(ROOT) as (api, _, _):
                for name, value in (("ksss-adoption.json", adoption(api, args.explicit_time)),
                                    ("effective-policy.json", resolve(api, args.explicit_time))):
                    (KSSS / name).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            result = {"materialization": "PASS"}
        elif args.action == "check":
            result = check(args.explicit_time)
        elif args.action == "knowledge":
            if not args.context:
                raise ConsumerError("KNOWLEDGE_CONTEXT_REQUIRED")
            result = knowledge_lookup(args.context)
        elif args.action == "evaluate":
            if not args.base:
                raise ConsumerError("BASE_REQUIRED")
            result = evaluate(args.base, args.explicit_time)
        else:
            if not all((args.receipt, args.artifact, args.evidence_root, args.environment_fingerprint)):
                raise ConsumerError("RECEIPT_ARTIFACT_EVIDENCE_ROOT_AND_ENVIRONMENT_REQUIRED")
            result = release_check(args.receipt, args.artifact, args.evidence_root, args.environment_fingerprint)
        payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(payload, encoding="utf-8")
        print(payload, end="")
        return 0
    except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError) as exc:
        print(json.dumps({"status": "FAIL", "reason": str(exc)}, sort_keys=True))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
