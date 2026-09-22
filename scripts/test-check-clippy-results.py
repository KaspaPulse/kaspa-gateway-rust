#!/usr/bin/env python3
"""Regression tests for the exact production Clippy diagnostic policy."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-clippy-results.py")


def diagnostic(path: str, line: int, code: str, *, level: str = "warning", message: str = "fixture") -> str:
    return json.dumps(
        {
            "reason": "compiler-message",
            "message": {
                "level": level,
                "message": message,
                "code": {"code": code, "explanation": None},
                "spans": [
                    {
                        "file_name": path,
                        "line_start": line,
                        "line_end": line,
                        "column_start": 1,
                        "column_end": 2,
                        "is_primary": True,
                    }
                ],
            },
        },
        separators=(",", ":"),
    )


COLLAPSIBLE = diagnostic(
    "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
    4079,
    "clippy::collapsible_if",
)
TOO_MANY = diagnostic(
    "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
    1841,
    "clippy::too_many_arguments",
)


def run(lines: list[str]) -> subprocess.CompletedProcess[str]:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as handle:
        handle.write("\n".join(lines))
        handle.write("\n")
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
    expect(0, run([]), "clean production diagnostics")
    expect(0, run([COLLAPSIBLE]), "exact collapsible-if")
    expect(0, run([TOO_MANY]), "exact too-many-arguments")
    expect(0, run([COLLAPSIBLE, TOO_MANY]), "both reviewed diagnostics")
    expect(
        1,
        run([diagnostic(
            "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
            4080,
            "clippy::collapsible_if",
        )]),
        "line drift fails closed",
    )
    expect(
        1,
        run([diagnostic(
            "apps/kaspa-gateway-desktop/src-tauri/src/other.rs",
            4079,
            "clippy::collapsible_if",
        )]),
        "path drift fails closed",
    )
    expect(
        1,
        run([diagnostic(
            "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
            4079,
            "clippy::needless_borrow",
        )]),
        "lint drift fails closed",
    )
    expect(
        0,
        run([diagnostic("crates/kaspa-gateway-rk-bridge/src/observation.rs", 30, "dead_code")]),
        "exact bridge dead-code fingerprint",
    )
    expect(
        0,
        run([diagnostic(
            "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
            3848,
            "dead_code",
        )]),
        "exact desktop preview dead-code fingerprint",
    )
    expect(
        0,
        run([
            diagnostic("crates/kaspa-gateway-rk-bridge/src/observation.rs", 30, "dead_code"),
            diagnostic("crates/kaspa-gateway-rk-bridge/src/observation.rs", 30, "dead_code"),
        ]),
        "duplicate exact bridge fingerprint",
    )
    expect(
        1,
        run([diagnostic("crates/kaspa-gateway-rk-bridge/src/observation.rs", 31, "dead_code")]),
        "dead-code line drift fails closed",
    )
    expect(
        1,
        run([diagnostic("crates/kaspa-gateway-rk-bridge/src/other.rs", 30, "dead_code")]),
        "dead-code path drift fails closed",
    )
    expect(
        1,
        run([diagnostic("apps/kaspa-gateway-desktop/src-tauri/src/lib.rs", 1, "E0308", level="error")]),
        "compiler error fails closed",
    )
    expect(1, run(["not-json"]), "invalid JSON fails closed")
    print("Production Clippy exact-diagnostic policy regression tests PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
