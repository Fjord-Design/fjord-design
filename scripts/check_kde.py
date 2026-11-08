"""Build and capture isolated KDE/Qt colour previews with existing native tools."""
import argparse
import configparser
from hashlib import sha256
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FIELDS = {"Window": ("Window", "BackgroundNormal"), "WindowText": ("Window", "ForegroundNormal"),
          "Base": ("View", "BackgroundNormal"), "Text": ("View", "ForegroundNormal"),
          "Button": ("Button", "BackgroundNormal"), "ButtonText": ("Button", "ForegroundNormal"),
          "Highlight": ("Selection", "BackgroundNormal"), "HighlightedText": ("Selection", "ForegroundNormal"),
          "Link": ("View", "ForegroundLink")}


def verify_palette(result, scheme):
    config = configparser.ConfigParser()
    config.read(scheme)
    for name, (group, field) in FIELDS.items():
        expected = "#" + "".join(f"{int(value):02x}" for value in config[f"Colors:{group}"][field].split(","))
        if result["colors"][name] != expected:
            raise ValueError(f"native KDE palette mismatch: {name}: {result['colors'][name]} != {expected}")
    if not result["focus"] or not result["selectedText"]:
        raise ValueError("native KDE preview lost selection or focus")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=ROOT / "dist/verification/kde-native")
    args = parser.parse_args()
    for tool in ("cmake", "c++"):
        if not shutil.which(tool):
            raise SystemExit(f"NOT RUN: missing existing {tool}; nothing installed")
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    build = output / "build"
    paths = [Path.home() / name for name in (".config/kdeglobals", ".config/kwinrc", ".config/plasmarc")]
    def snapshot():
        return {path.name: sha256(path.read_bytes()).hexdigest() if path.is_file() else None for path in paths}
    before = snapshot()
    results = {}
    try:
        subprocess.run(["cmake", "-S", str(ROOT / "adapters/kde"), "-B", str(build)], check=True)
        subprocess.run(["cmake", "--build", str(build)], check=True)
        for identity in ("fjord", "work", "gaming"):
            folder = ROOT / "dist/adapters/kde" if identity == "fjord" else ROOT / f"dist/variants/{identity}/kde"
            for mode in ("Night", "Day", "Aurora"):
                scheme = folder / f"Fjord{mode}.colors"
                prefix = output / f"{identity}-{mode}"
                subprocess.run([str(build / "fjord-kde-preview"), str(scheme), str(prefix)], check=True, timeout=20)
                result = json.loads(prefix.with_suffix(".json").read_text())
                verify_palette(result, scheme)
                for suffix in ("-before.png", "-after.png"):
                    if not Path(str(prefix) + suffix).is_file():
                        raise ValueError("native KDE screenshot is missing")
                results[f"{identity}-{mode}"] = result
    finally:
        integrity = {"before": before, "after": snapshot(), "unchanged": before == snapshot()}
        (output / "settings-integrity.json").write_text(json.dumps(integrity, indent=2) + "\n")
        if not integrity["unchanged"]:
            raise ValueError("desktop settings changed during preview; inspect without overwriting user changes")
    (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    print("PASS nine isolated KDE/Qt native palettes, focus and screenshots; checked settings unchanged")


if __name__ == "__main__":
    main()
