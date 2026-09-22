#!/usr/bin/env python3
"""Regression tests for the exact TruffleHog historical false-positive policy."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-trufflehog-results.py")
BASE = {
    "SourceMetadata": {
        "Data": {
            "Git": {
                "commit": "8f209ba516707b11098bd962972da38157346833",
                "file": "crates/kaspa-gateway-security/src/lib.rs",
                "line": 374,
            }
        }
    },
    "DetectorName": "URI",
    "DecoderName": "PLAIN",
    "Verified": False,
}
LOB_FALSE_POSITIVE = {
    "SourceMetadata": {
        "Data": {
            "Git": {
                "commit": "d079d38c8a78de400a5d6b2d06819feb1ff73df9",
                "file": ".security/ksss/test_consumer.py",
                "line": 115,
            }
        }
    },
    "DetectorName": "Lob",
    "DecoderName": "PLAIN",
    "Verified": True,
    "Raw": "test_diagnosis_requires_knowledge_search",
}


def run(results: list[object], raw_tail: str = "") -> subprocess.CompletedProcess[str]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as handle:
        for result in results:
            handle.write(json.dumps(result) + "\n")
        handle.write(raw_tail)
        path = Path(handle.name)
    try:
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(path)],
            text=True,
            capture_output=True,
            check=False,
        )
    finally:
        path.unlink(missing_ok=True)


def changed(**updates: object) -> dict:
    item = json.loads(json.dumps(BASE))
    git = item["SourceMetadata"]["Data"]["Git"]
    for key, value in updates.items():
        if key.startswith("git_"):
            git[key[4:]] = value
        else:
            item[key] = value
    return item


def lob_changed(**updates: object) -> dict:
    item = json.loads(json.dumps(LOB_FALSE_POSITIVE))
    git = item["SourceMetadata"]["Data"]["Git"]
    for key, value in updates.items():
        if key.startswith("git_"):
            git[key[4:]] = value
        else:
            item[key] = value
    return item


def expect_exit(expected: int, result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode != expected:
        raise AssertionError(
            f"{label}: expected exit {expected}, got {result.returncode}; "
            f"stdout={result.stdout!r} stderr={result.stderr!r}"
        )


def main() -> int:
    squash = changed(git_commit="3f6fb666241135be0f6f5071994bcf4deeb75326", git_line=376)
    expect_exit(0, run([]), "empty result set")
    expect_exit(0, run([BASE]), "exact original historical false positive")
    expect_exit(0, run([squash]), "exact squash historical false positive")
    expect_exit(0, run([LOB_FALSE_POSITIVE]), "exact raw-bound Lob false positive")
    expect_exit(0, run([BASE, squash, LOB_FALSE_POSITIVE]), "all exact historical false positives")
    expect_exit(1, run([BASE, BASE]), "duplicate original historical exception")
    expect_exit(1, run([squash, squash]), "duplicate squash historical exception")
    expect_exit(1, run([LOB_FALSE_POSITIVE, LOB_FALSE_POSITIVE]), "duplicate Lob historical exception")
    expect_exit(1, run([changed(git_commit="deadbeef")]), "commit drift")
    expect_exit(1, run([changed(git_file="other.rs")]), "path drift")
    expect_exit(1, run([changed(git_line=375)]), "line drift")
    expect_exit(1, run([changed(DetectorName="Generic")]), "detector drift")
    expect_exit(1, run([changed(DecoderName="BASE64")]), "decoder drift")
    expect_exit(1, run([changed(Verified=True)]), "verified secret cannot be allowed generically")
    expect_exit(1, run([lob_changed(Raw="test_diagnosis_requires_knowledge_search_x")]), "Lob raw drift")
    expect_exit(1, run([lob_changed(git_commit="deadbeef")]), "Lob commit drift")
    expect_exit(1, run([lob_changed(git_line=116)]), "Lob line drift")
    expect_exit(1, run([lob_changed(DetectorName="Generic")]), "Lob detector drift")
    expect_exit(1, run([], raw_tail="not-json\n"), "malformed scanner output")
    print("TruffleHog exact historical false-positive policy regression tests PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
