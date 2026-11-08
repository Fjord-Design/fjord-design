"""Build the complete-library local candidate with native packages; never publish."""
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from fjord_tokens.cli import ROOT, write_binary_output
from fjord_tokens.project import metadata
from package_candidate import files, dependency_notices

VERSION = "1.0.0-rc.1"
PLATFORMS = ("wezterm", "zed", "vscode", "neovim", "kde")


def bundle(root):
    root = Path(root)
    contents = dict(files(root))
    contents.update(dependency_notices(root))
    for folder, prefix in (("dist/adapters", "platforms"), ("dist/variants", "variants")):
        directory = root / folder
        if not directory.is_dir():
            raise ValueError(f"missing library output: {folder}")
        for path in sorted(directory.rglob("*")):
            if path.is_symlink():
                raise ValueError(f"symlink in library output: {path}")
            if path.is_file():
                contents[f"{prefix}/{path.relative_to(directory).as_posix()}"] = path.read_bytes()
    for platform in PLATFORMS:
        if f"platforms/{platform}/report.json" not in contents:
            raise ValueError(f"missing platform report: {platform}")
    if json.loads(contents["svelte/package.json"])["version"] != VERSION:
        raise ValueError("library and Svelte candidate versions differ")
    contents["platforms/vscode/fjord-themes-0.1.0.vsix"] = (root / "dist/release/fjord-themes-0.1.0.vsix").read_bytes()
    manifest = {"status": "candidate", "libraryVersion": VERSION,
                "svelteVersion": VERSION, "tokenContract": metadata()["contractVersion"],
                "componentCount": 66, "platforms": list(PLATFORMS),
                "supportMatrix": "docs/support-matrix.md",
                "sha256": {name: sha256(value).hexdigest() for name, value in sorted(contents.items())}}
    contents["manifest.json"] = (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode()
    target = BytesIO()
    with ZipFile(target, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for name, value in sorted(contents.items()):
            info = ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, value, compresslevel=9)
    return target.getvalue()


if __name__ == "__main__":
    target = ROOT / "dist/release" / f"fjord-library-{VERSION}.zip"
    first = bundle(ROOT)
    if first != bundle(ROOT):
        raise ValueError("library archive is not reproducible")
    write_binary_output(target, first, [ROOT / folder for folder in ("tokens", "src", "docs", "assets", "tests", "reference", "scripts", ".git")])
    write_binary_output(target.with_suffix(".zip.sha256"), (sha256(first).hexdigest() + "  " + target.name + "\n").encode(), [])
    print(f"Prepared reproducible local library candidate: {target}")
