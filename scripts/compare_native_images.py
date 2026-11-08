"""Compare reviewed native screenshots on the same OS/font/toolchain; installs nothing."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess


def compare(baseline, actual):
    expected = {path.relative_to(baseline) for path in baseline.rglob("*.png")}
    found = {path.relative_to(actual) for path in actual.rglob("*.png")}
    if not expected or expected != found:
        raise ValueError("screenshot membership differs or baseline has no PNG files")
    differences = {}
    for name in sorted(expected):
        result = subprocess.run(["magick", "compare", "-metric", "AE", str(baseline / name), str(actual / name), "null:"], capture_output=True, text=True, timeout=20)
        if result.returncode not in (0, 1):
            raise ValueError(f"cannot compare {name}: {result.stderr}")
        differences[name.as_posix()] = float(result.stderr.strip().split()[0])
    return differences


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("actual", type=Path)
    args = parser.parse_args()
    if not shutil.which("magick"):
        raise SystemExit("NOT RUN: no existing ImageMagick runtime; nothing installed")
    differences = compare(args.baseline, args.actual)
    print(json.dumps(differences, indent=2))
    if any(differences.values()):
        raise SystemExit("FAIL changed pixels; review target/font changes before updating the baseline")
    print("PASS native screenshots match the reviewed baseline")
