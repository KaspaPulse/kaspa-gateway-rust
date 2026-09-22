#!/usr/bin/env python3
"""Accept only exact, reviewed Clippy diagnostics from the Desktop production crate."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ANSI = re.compile(r"\x1b\[[0-9;]*m")
ERROR = re.compile(r"^\s*error(?:\[[^\]]+\])?:\s*(?P<message>.*)$")
LOCATION = re.compile(r"^\s*-->\s+(?P<path>.+?):(?P<line>\d+):(?P<column>\d+)\s*$")
LINT = re.compile(r"`-D\s+(?P<lint>clippy::[a-z0-9_-]+)`")
SUMMARY_PREFIXES = (
    "could not compile ",
    "aborting due to ",
)

ALLOWED = {
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        4079,
        "clippy::collapsible-if",
    ),
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        1841,
        "clippy::too-many-arguments",
    ),
}


def normalized_lines(text: str) -> list[str]:
    return [ANSI.sub("", line) for line in text.splitlines()]


def error_blocks(text: str) -> list[list[str]]:
    blocks: list[list[str]] = []
    current: list[str] | None = None
    for line in normalized_lines(text):
        if ERROR.match(line):
            if current is not None:
                blocks.append(current)
            current = [line]
        elif current is not None:
            current.append(line)
    if current is not None:
        blocks.append(current)
    return blocks


def fingerprint(block: list[str]) -> tuple[str, int, str] | None:
    header = ERROR.match(block[0])
    assert header is not None
    message = header.group("message").strip()
    if message.startswith(SUMMARY_PREFIXES):
        return None

    location = None
    lint = None
    for line in block:
        if location is None:
            match = LOCATION.match(line)
            if match:
                location = (match.group("path").replace("\\", "/"), int(match.group("line")))
        if lint is None:
            match = LINT.search(line)
            if match:
                lint = match.group("lint")
    if location is None or lint is None:
        return ("<unclassified>", -1, message)
    return (location[0], location[1], lint)


def evaluate(path: Path) -> tuple[int, list[tuple[str, int, str]]]:
    accepted = 0
    unexpected: list[tuple[str, int, str]] = []
    for block in error_blocks(path.read_text(encoding="utf-8", errors="replace")):
        current = fingerprint(block)
        if current is None:
            continue
        if current in ALLOWED:
            accepted += 1
        else:
            unexpected.append(current)
    return accepted, unexpected


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check-clippy-results.py <clippy-log>", file=sys.stderr)
        return 2
    try:
        accepted, unexpected = evaluate(Path(sys.argv[1]))
    except OSError as error:
        print(f"clippy-result-policy: FAIL: {error}", file=sys.stderr)
        return 1
    if unexpected:
        print(
            f"clippy-result-policy: FAIL: {len(unexpected)} unexpected error(s)",
            file=sys.stderr,
        )
        for item in unexpected:
            print(f"  path={item[0]} line={item[1]} lint={item[2]}", file=sys.stderr)
        return 1
    if accepted == 0:
        print("clippy-result-policy: FAIL: no reviewed diagnostics found", file=sys.stderr)
        return 1
    print(
        "clippy-result-policy: PASS "
        f"(reviewed diagnostics accepted={accepted}; unexpected=0)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
