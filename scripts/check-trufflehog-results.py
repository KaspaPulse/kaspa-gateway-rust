#!/usr/bin/env python3
"""Fail closed on TruffleHog findings with exact historical false positives."""

import json
import sys
from pathlib import Path

ALLOWED_HISTORICAL_FALSE_POSITIVES = {
    ("8f209ba516707b11098bd962972da38157346833", "crates/kaspa-gateway-security/src/lib.rs", 374, "URI", "PLAIN", False),
    ("3f6fb666241135be0f6f5071994bcf4deeb75326", "crates/kaspa-gateway-security/src/lib.rs", 376, "URI", "PLAIN", False),
}

ALLOWED_EXACT_VERIFIED_FALSE_POSITIVES = {
    (
        "d079d38c8a78de400a5d6b2d06819feb1ff73df9",
        ".security/ksss/test_consumer.py",
        115,
        "Lob",
        "PLAIN",
        True,
        "test_diagnosis_requires_knowledge_search",
    ),
}


def fingerprint(result: dict) -> tuple[object, ...]:
    git = (
        result.get("SourceMetadata", {})
        .get("Data", {})
        .get("Git", {})
    )
    return (
        git.get("commit"),
        git.get("file"),
        git.get("line"),
        result.get("DetectorName"),
        result.get("DecoderName"),
        result.get("Verified"),
    )


def evaluate(path: Path) -> tuple[int, int, list[tuple[object, ...]]]:
    seen_allowed: set[tuple[object, ...]] = set()
    unexpected: list[tuple[object, ...]] = []
    with path.open("r", encoding="utf-8-sig") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip():
                continue
            try:
                result = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError(
                    f"invalid TruffleHog JSON on output line {line_number}: {error.msg}"
                ) from error
            if not isinstance(result, dict):
                raise ValueError(
                    f"invalid TruffleHog result type on output line {line_number}"
                )
            current = fingerprint(result)
            exact_current = (*current, result.get("Raw"))
            allowed = current if current in ALLOWED_HISTORICAL_FALSE_POSITIVES else None
            if exact_current in ALLOWED_EXACT_VERIFIED_FALSE_POSITIVES:
                allowed = exact_current
            if allowed is not None:
                if allowed in seen_allowed:
                    raise ValueError("historical false-positive result appeared more than once")
                seen_allowed.add(allowed)
                continue
            unexpected.append(current)
    return len(seen_allowed), len(unexpected), unexpected


def describe(item: tuple[object, ...]) -> str:
    commit, file_name, line, detector, decoder, verified = item
    return (
        f"commit={commit!s} file={file_name!s} line={line!s} "
        f"detector={detector!s} decoder={decoder!s} verified={verified!s}"
    )


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-trufflehog-results.py <jsonl>", file=sys.stderr)
        return 2
    try:
        allowed_count, unexpected_count, unexpected = evaluate(Path(sys.argv[1]))
    except (OSError, ValueError) as error:
        print(f"trufflehog-result-policy: FAIL: {error}", file=sys.stderr)
        return 1
    if unexpected_count:
        print(
            f"trufflehog-result-policy: FAIL: {unexpected_count} unexpected finding(s)",
            file=sys.stderr,
        )
        for item in unexpected:
            print(f"  {describe(item)}", file=sys.stderr)
        return 1
    print(
        "trufflehog-result-policy: PASS "
        f"(exact historical false positives accepted={allowed_count}; unexpected=0)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
