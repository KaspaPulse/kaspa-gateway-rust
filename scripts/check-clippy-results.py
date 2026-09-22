#!/usr/bin/env python3
"""Fail closed on production Clippy diagnostics except two exact reviewed Desktop lints."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ALLOWED = {
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        4079,
        "clippy::collapsible_if",
    ),
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        1841,
        "clippy::too_many_arguments",
    ),
    (
        "crates/kaspa-gateway-rk-bridge/src/observation.rs",
        30,
        "dead_code",
    ),
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        3848,
        "dead_code",
    ),
}


def primary_fingerprint(message: dict) -> tuple[str, int, str]:
    code = ((message.get("code") or {}).get("code") or "<no-code>").replace("-", "_")
    spans = message.get("spans") or []
    primary = next((span for span in spans if span.get("is_primary")), None)
    if primary is None:
        return ("<unclassified>", -1, code)
    path = str(primary.get("file_name") or "<unknown>").replace("\\", "/")
    line = int(primary.get("line_start") or -1)
    return (path, line, code)


def evaluate(path: Path) -> tuple[int, list[tuple[str, int, str, str]]]:
    accepted = 0
    unexpected: list[tuple[str, int, str, str]] = []
    for line_number, raw in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1):
        if not raw.strip():
            continue
        try:
            record = json.loads(raw)
        except json.JSONDecodeError as error:
            unexpected.append(("<invalid-json>", line_number, "<parse>", str(error)))
            continue
        if record.get("reason") != "compiler-message":
            continue
        message = record.get("message") or {}
        level = str(message.get("level") or "")
        if level not in {"warning", "error"}:
            continue
        fingerprint = primary_fingerprint(message)
        rendered = str(message.get("message") or "")
        if fingerprint in ALLOWED and level == "warning":
            accepted += 1
            continue
        unexpected.append((fingerprint[0], fingerprint[1], fingerprint[2], f"{level}: {rendered}"))
    return accepted, unexpected


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-clippy-results.py <cargo-clippy-jsonl>", file=sys.stderr)
        return 2
    try:
        accepted, unexpected = evaluate(Path(sys.argv[1]))
    except OSError as error:
        print(f"clippy-result-policy: FAIL: {error}", file=sys.stderr)
        return 1
    if unexpected:
        print(f"clippy-result-policy: FAIL: {len(unexpected)} unexpected diagnostic(s)", file=sys.stderr)
        for path, line, code, message in unexpected:
            print(f"  path={path} line={line} code={code} message={message}", file=sys.stderr)
        return 1
    print(
        "clippy-result-policy: PASS "
        f"(reviewed diagnostics accepted={accepted}; unexpected=0)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
