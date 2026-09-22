#!/usr/bin/env python3
"""Regression tests for the exact Desktop Clippy exception policy."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-clippy-results.py")

COLLAPSIBLE = """error: this `if` statement can be collapsed
  --> apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs:4079:5
   |
   = note: `-D clippy::collapsible-if` implied by `-D warnings`
error: could not compile `kaspa-gateway-desktop` (lib) due to 1 previous error
"""

TOO_MANY = """error: this function has too many arguments (8/7)
  --> apps/kaspa-gateway-desktop/src-tauri/src/lib.rs:1841:1
   |
   = note: `-D clippy::too-many-arguments` implied by `-D warnings`
error: could not compile `kaspa-gateway-desktop` (lib) due to 1 previous error
"""


def run(text: str) -> subprocess.CompletedProcess[str]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as handle:
        handle.write(text)
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


def expect(code: int, result: subprocess.CompletedProcess[str], label: str) -> None:
    if result.returncode != code:
        raise AssertionError(
            f"{label}: expected={code} actual={result.returncode} "
            f"stdout={result.stdout!r} stderr={result.stderr!r}"
        )


def main() -> int:
    expect(0, run(COLLAPSIBLE), "exact collapsible-if")
    expect(0, run(TOO_MANY), "exact too-many-arguments")
    expect(0, run(COLLAPSIBLE + TOO_MANY), "both reviewed diagnostics")
    expect(
        1,
        run(COLLAPSIBLE.replace(":4079:5", ":4080:5")),
        "line drift fails closed",
    )
    expect(
        1,
        run(COLLAPSIBLE.replace("integrated_runtime_commands.rs", "other.rs")),
        "path drift fails closed",
    )
    expect(
        1,
        run(COLLAPSIBLE.replace("clippy::collapsible-if", "clippy::needless-borrow")),
        "lint drift fails closed",
    )
    expect(
        1,
        run("error[E0308]: mismatched types\n  --> src/lib.rs:1:1\n"),
        "rustc error fails closed",
    )
    expect(1, run(""), "empty log fails closed")
    print("Desktop Clippy exact-exception policy regression tests PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
