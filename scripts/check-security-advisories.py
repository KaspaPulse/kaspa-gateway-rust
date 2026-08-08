#!/usr/bin/env python3
"""Fail CI when RustSec ignores are undocumented, inconsistent, or stale."""

from __future__ import annotations

import argparse
import re
from datetime import date, datetime, timezone
from pathlib import Path

RUSTSEC_PATTERN = re.compile(r"RUSTSEC-\d{4}-\d{4}")
REVIEW_PATTERN = re.compile(r"Last automated review:\s*\*\*(\d{4}-\d{2}-\d{2})\*\*")


def fail(message: str) -> None:
    print(f"security-advisory-check: {message}", file=__import__("sys").stderr)
    raise SystemExit(1)


def ids(path: Path) -> set[str]:
    return set(RUSTSEC_PATTERN.findall(path.read_text(encoding="utf-8")))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-age-days", type=int, default=45)
    args = parser.parse_args()

    if args.max_age_days < 1:
        fail("--max-age-days must be positive")

    audit_path = Path(".cargo/audit.toml")
    deny_path = Path("deny.toml")
    report_path = Path("SECURITY_ADVISORIES.md")

    for path in (audit_path, deny_path, report_path):
        if not path.is_file():
            fail(f"required policy file is missing: {path}")

    audit_ids = ids(audit_path)
    deny_ids = ids(deny_path)
    report_ids = ids(report_path)

    if audit_ids != deny_ids:
        fail(
            "cargo-audit and cargo-deny ignore sets differ: "
            f"audit={sorted(audit_ids)} deny={sorted(deny_ids)}"
        )

    undocumented = sorted(audit_ids - report_ids)
    if undocumented:
        fail("undocumented ignored RustSec IDs: " + ", ".join(undocumented))

    report = report_path.read_text(encoding="utf-8")
    match = REVIEW_PATTERN.search(report)
    if not match:
        fail("SECURITY_ADVISORIES.md is missing 'Last automated review: **YYYY-MM-DD**'")

    try:
        reviewed = date.fromisoformat(match.group(1))
    except ValueError as exc:
        fail(f"invalid review date: {exc}")

    today = datetime.now(timezone.utc).date()
    age_days = (today - reviewed).days
    if age_days < 0:
        fail(f"review date {reviewed.isoformat()} is in the future")
    if age_days > args.max_age_days:
        fail(
            f"security advisory review is {age_days} days old; "
            f"maximum allowed age is {args.max_age_days} days"
        )

    print(
        "security-advisory-check: PASS "
        f"({len(audit_ids)} managed RustSec IDs; review age {age_days} days)"
    )


if __name__ == "__main__":
    main()
