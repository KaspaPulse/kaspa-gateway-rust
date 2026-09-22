#!/usr/bin/env python3
"""One offline KGW adoption gate; does not build or run the application."""
import subprocess
import sys
from pathlib import Path


def main():
    if sys.version_info < (3, 12):
        print("KSSS v1.2.0 requires Python >=3.12", file=sys.stderr)
        return 1
    root = Path(__file__).resolve().parents[1]
    for args in (
        [".security/ksss/consumer.py", "check"],
        ["-m", "unittest", "discover", "-s", ".security/ksss", "-p", "test_*.py", "-v"],
    ):
        result = subprocess.run([sys.executable, "-B", *args], cwd=root, check=False)
        if result.returncode:
            return result.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
