"""Record pinned font-file metrics with the installed Fontconfig tools."""

import ctypes
import ctypes.util
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[1]
PROBE = "".join(chr(i) for i in range(32, 127)) + "æøåÆØÅéÉ"


def main():
    directory = ROOT / "assets/fonts"
    manifest = json.loads((directory / "manifest.json").read_text())
    library_path = ctypes.util.find_library("fontconfig")
    if library_path is None and shutil.which("fc-scan"):
        sibling = Path(shutil.which("fc-scan")).resolve().parent.parent / "lib/libfontconfig.1.dylib"
        if sibling.is_file():
            library_path = str(sibling)
    if library_path is None:
        raise RuntimeError("Fontconfig is required for font-file inspection")
    library = ctypes.CDLL(library_path)
    weight = library.FcWeightToOpenTypeDouble
    weight.argtypes = [ctypes.c_double]
    weight.restype = ctypes.c_double
    faces = []
    for family in manifest["families"]:
        for face in family["faces"]:
            source = directory / face["file"]
            output = subprocess.check_output(["fc-scan", "--format",
                                             "%{family}|%{style}|%{variable}|%{weight}|%{slant}|%{fontversion}|%{charset}\n",
                                             str(source)], text=True)
            entries = [line.split("|") for line in output.splitlines()]
            entry = next((entry for entry in entries if entry[2] == "True"), entries[0])
            names, style, variable, weights, slant, version, charset = entry
            weights = [round(weight(float(v))) for v in re.findall(r"[0-9.]+", weights)]
            minimum, maximum = min(weights), max(weights)
            ranges = [tuple(int(v, 16) for v in item.split("-")) for item in charset.split()]
            missing = [f"U+{ord(char):04X}" for char in PROBE
                       if not any(first <= ord(char) <= values[-1] for values in ranges for first in values[:1])]
            assert any(name == family["family"] or name.startswith(family["family"] + " ")
                       for name in names.split(",")), (face["file"], names)
            assert [minimum, maximum] == face["weight"], (face["file"], weights)
            assert not missing, (face["file"], missing)
            assert (int(slant) != 0) == (face["style"] == "italic"), (face["file"], slant)
            faces.append({"file": face["file"], "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                          "family": family["family"], "reportedFamily": names, "style": face["style"], "variable": variable == "True",
                          "weight": [minimum, maximum], "fontVersionFixed": int(version),
                          "missingProbeCodepoints": missing})
    report = {"tool": subprocess.check_output(["fc-scan", "--version"], stderr=subprocess.STDOUT, text=True).strip(),
              "probe": PROBE, "faces": faces}
    (directory / "inspection.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(f"PASS inspected {len(faces)} WOFF2 files: family, style, variable weights and {len(PROBE)} glyphs")


if __name__ == "__main__":
    main()
