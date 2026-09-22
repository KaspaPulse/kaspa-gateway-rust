"""Offline contract tests. Synthetic receipts never represent an application run."""
import copy
import json
import socket
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import consumer
import runtime_loader

NOW = "2026-09-20T15:00:00+00:00"


class ConsumerContracts(unittest.TestCase):
    def test_offline_self_contained_check(self):
        with patch.object(socket.socket, "connect", side_effect=AssertionError("network forbidden")), \
             patch.object(socket, "create_connection", side_effect=AssertionError("network forbidden")):
            result = consumer.check(NOW)
        self.assertEqual(result["adoption_status"], "PASS")
        self.assertFalse(result["external_ksss_checkout_required"])
        self.assertEqual(result["application_runtime_status"], "NOT_VERIFIED")
        self.assertEqual(result["release_qualification"], "NOT_VERIFIED")

    def test_mixed_changes_preserve_every_official_trigger(self):
        with runtime_loader.runtime_context() as (api, _, _):
            self.assertEqual(consumer.classify(api, [
                "crates/kaspa-gateway-runtime/src/lib.rs", "Cargo.lock",
                ".github/workflows/ci.yml", ".security/ksss/trust-policy.json"
            ]), ["CODE_ONLY_CHANGE", "DEPENDENCY_CHANGE", "POLICY_CHANGE", "WORKFLOW_CHANGE"])

    def test_risk_floor_cannot_be_lowered_by_profile(self):
        original = consumer.load
        def weakened(name):
            value = copy.deepcopy(original(name))
            if name == "repository-policy.yaml":
                value["audit_profile"] = "light"
                value["controls"]["additional_required"] = []
            return value
        with runtime_loader.runtime_context() as (api, _, _), patch.object(consumer, "load", weakened):
            with self.assertRaisesRegex(consumer.ConsumerError, "RISK_FLOOR_WEAKENED"):
                consumer.resolve(api, NOW)

    def test_signed_archive_pin_rejects_modified_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            directory = repo / ".security/ksss/trust/evidence/v1.2.0"
            directory.mkdir(parents=True)
            policy = consumer.load("trust-policy.json")
            (repo / ".security/ksss/trust-policy.json").write_text(json.dumps(policy))
            (directory / policy["runtime_artifact_filename"]).write_bytes(b"modified")
            with self.assertRaisesRegex(runtime_loader.RuntimeLoaderError, "SHA256_MISMATCH"):
                runtime_loader.verify_artifact_pin(repo)

    def test_runtime_rollback_floor_rejected(self):
        policy = copy.deepcopy(consumer.load("trust-policy.json"))
        policy["runtime_sequence"] = 1
        with patch.object(runtime_loader, "load_policy", return_value=policy):
            with self.assertRaisesRegex(runtime_loader.RuntimeLoaderError, "BELOW_FLOOR"):
                runtime_loader.verify_artifact_pin()

    def test_archive_paths_fail_closed(self):
        for value in ("../bad", "ksss-consumer-runtime-v1/../bad", "ksss-consumer-runtime-v1/"):
            with self.subTest(value=value), self.assertRaises(runtime_loader.RuntimeLoaderError):
                runtime_loader._relative_member(value)

    def test_learning_rejects_stale_and_wrong_context(self):
        record = consumer.load("learning/known-failures/KGW-OWNER-TERMINAL-PID.json")
        with runtime_loader.runtime_context() as (api, _, _):
            self.assertEqual(len(api.lookup_known_failures(
                [record], fingerprint=record["failure_fingerprint"], context=record["applies_when"]
            )), 1)
            context = {**record["applies_when"], "platform": "linux"}
            self.assertEqual(api.lookup_known_failures(
                [record], fingerprint=record["failure_fingerprint"], context=context), [])
            self.assertFalse(api.record_reusable(
                record, context=record["applies_when"], active_change_types=["CODE_ONLY_CHANGE"]))

    def test_reference_parity_rejects_different_signed_runtime(self):
        original = consumer.load
        def mismatched(name):
            value = copy.deepcopy(original(name))
            if name == "reference-parity.json":
                value["runtime_artifact_sha256"] = "f" * 64
            return value
        with runtime_loader.runtime_context() as (_, _, manifest), patch.object(consumer, "load", mismatched):
            with self.assertRaisesRegex(consumer.ConsumerError, "PARITY_IDENTITY"):
                consumer.reference_parity(manifest)

    def test_threat_model_rejects_missing_surface_inventory(self):
        original = consumer.load
        def incomplete(name):
            value = copy.deepcopy(original(name))
            if name == "threat-model.json":
                value.pop("surfaces")
            return value
        with patch.object(consumer, "load", incomplete):
            with self.assertRaisesRegex(consumer.ConsumerError, "THREAT_MODEL_INVALID"):
                consumer.check(NOW)

    def test_knowledge_lookup_reads_existing_evidence_without_reexecution(self):
        record = consumer.load("learning/known-failures/KGW-OWNER-TERMINAL-PID.json")
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "query.json"
            path.write_text(json.dumps({"context": record["applies_when"],
                                       "failure_fingerprint": record["failure_fingerprint"]}))
            result = consumer.knowledge_lookup(path)
        self.assertEqual(result["PREVIOUS_KNOWLEDGE_SEARCH"], "PASS")
        self.assertEqual(result["match_status"], "MATCH")
        self.assertFalse(result["historical_tests_reexecuted"])

    def test_diagnosis_requires_knowledge_search(self):
        with runtime_loader.runtime_context() as (api, _, _):
            with self.assertRaises(ValueError):
                api.validate_diagnosis_sequence(["DIAGNOSIS"])
            api.validate_diagnosis_sequence(
                ["CONTAIN_OR_RESTORE", "PREVIOUS_KNOWLEDGE_SEARCH=PASS", "DIAGNOSIS"],
                high_severity_production_incident=True)

    def test_learning_cannot_close_without_protection(self):
        with runtime_loader.runtime_context() as (api, _, _):
            with self.assertRaises(ValueError):
                api.learning_closeout(service_status="RESOLVED", learning_status="CLOSED", durable_protections=[])


class RuntimeReceiptContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.artifact = self.root / "candidate.exe"
        self.artifact.write_bytes(b"SYNTHETIC TEST ARTIFACT")
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        proof = self.evidence / "proof.json"
        proof.write_text('{"synthetic_contract_test_only":true}')
        self.current = {"source_sha": "a" * 40, "tree_hash": "b" * 64, "worktree_state": {"fixture": "synthetic"}}
        contract = consumer.load("runtime-contract.json")
        self.receipt = {
            "schema": "kgw-runtime-qualification-receipt-v1", **self.current,
            "artifact_sha256": consumer.digest(self.artifact),
            "environment_fingerprint": "c" * 64, "host_os": "windows",
            "test_kind": "real-packaged-application",
            "cases": [{
                "case_id": case, "owner_identity": {"pid": 1234, "start_time": 123456, "executable": "C:/fixture/candidate.exe"},
                "checks": {name: {"status": "PASS", "evidence_path": "proof.json", "evidence_sha256": consumer.digest(proof)}
                           for name in contract["required_checks"]}
            } for case in contract["cases"]]
        }

    def verify(self):
        path = self.root / "receipt.json"
        path.write_text(json.dumps(self.receipt))
        with patch.object(consumer, "identity", return_value=self.current):
            return consumer.release_check(path, self.artifact, self.evidence, "c" * 64)

    def test_complete_synthetic_receipt_is_only_receipt_validation(self):
        result = self.verify()
        self.assertEqual(result["receipt_validation"], "PASS")
        self.assertFalse(result["application_executed_by_this_command"])

    def test_missing_case_rejected(self):
        self.receipt["cases"].pop()
        with self.assertRaisesRegex(consumer.ConsumerError, "CASES_INCOMPLETE"):
            self.verify()

    def test_duplicate_case_rejected(self):
        self.receipt["cases"][-1] = self.receipt["cases"][0]
        with self.assertRaisesRegex(consumer.ConsumerError, "CASES_INCOMPLETE"):
            self.verify()

    def test_ci_success_cannot_replace_runtime_proof(self):
        self.receipt["test_kind"] = "unit-tests"
        with self.assertRaisesRegex(consumer.ConsumerError, "REAL_PACKAGED_WINDOWS"):
            self.verify()

    def test_stale_source_rejected(self):
        self.receipt["source_sha"] = "d" * 40
        with self.assertRaisesRegex(consumer.ConsumerError, "IDENTITY_MISMATCH"):
            self.verify()

    def test_different_artifact_rejected(self):
        self.artifact.write_bytes(b"DIFFERENT")
        with self.assertRaisesRegex(consumer.ConsumerError, "ARTIFACT_MISMATCH"):
            self.verify()

    def test_different_environment_rejected(self):
        self.receipt["environment_fingerprint"] = "d" * 64
        with self.assertRaisesRegex(consumer.ConsumerError, "ENVIRONMENT_MISMATCH"):
            self.verify()

    def test_missing_owner_identity_rejected(self):
        self.receipt["cases"][0]["owner_identity"].pop("start_time")
        with self.assertRaisesRegex(consumer.ConsumerError, "OWNER_IDENTITY"):
            self.verify()

    def test_failed_check_rejected(self):
        self.receipt["cases"][0]["checks"]["ready_owned_process"]["status"] = "FAIL"
        with self.assertRaisesRegex(consumer.ConsumerError, "CHECK_NOT_PASS"):
            self.verify()

    def test_missing_evidence_rejected(self):
        self.receipt["cases"][0]["checks"]["ready_owned_process"]["evidence_path"] = "missing.json"
        with self.assertRaisesRegex(consumer.ConsumerError, "EVIDENCE_PATH"):
            self.verify()

    def test_evidence_tampering_rejected(self):
        (self.evidence / "proof.json").write_text("changed")
        with self.assertRaisesRegex(consumer.ConsumerError, "EVIDENCE_DIGEST"):
            self.verify()

    def test_evidence_path_escape_rejected(self):
        self.receipt["cases"][0]["checks"]["ready_owned_process"]["evidence_path"] = "../candidate.exe"
        with self.assertRaisesRegex(consumer.ConsumerError, "EVIDENCE_PATH"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
