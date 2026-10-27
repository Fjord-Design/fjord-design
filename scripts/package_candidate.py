"""Assemble a reproducible local MVP candidate, without publishing it."""

from hashlib import sha256
from io import BytesIO
import json
import os
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from fjord_tokens.cli import ROOT, write_binary_output
from fjord_tokens.project import metadata

VERSION = "0.1.0"
MODES = ("night", "day", "aurora")
SUFFIXES = ("dtcg.json", "resolved.json", "tokens.css", "wezterm.toml")


def dependency_notices(root):
    """Collect installed package metadata and root notices for later review."""
    cache = Path(root) / "node_modules/.bun"
    if not cache.is_dir():
        raise ValueError("missing Bun dependency store; run mise run setup")
    records, output = {}, {}
    for modules in sorted(cache.glob("*/node_modules")):
        for entry in sorted(modules.iterdir()):
            packages = sorted(entry.iterdir()) if entry.name.startswith("@") and entry.is_dir() and not entry.is_symlink() else [entry]
            for package in packages:
                if package.is_symlink() or not (package / "package.json").is_file():
                    continue
                raw = (package / "package.json").read_bytes()
                metadata_file = json.loads(raw)
                name, version = metadata_file.get("name"), metadata_file.get("version")
                if not isinstance(name, str) or not isinstance(version, str):
                    raise ValueError(f"invalid installed package metadata: {package}")
                key = sha256(raw).hexdigest()[:20]
                if key in records:
                    continue
                prefix = f"third-party-notices/{key}/"
                output[prefix + "package.json"] = raw
                notices = []
                for item in sorted(package.iterdir()):
                    if item.is_file() and not item.is_symlink() and item.name.lower().startswith(("license", "licence", "copying", "notice")):
                        output[prefix + item.name] = item.read_bytes()
                        notices.append(prefix + item.name)
                records[key] = {"name": name, "version": version, "declaredLicense": metadata_file.get("license"), "notices": notices}
    output["third-party-notices/index.json"] = (json.dumps({"packages": records}, ensure_ascii=False, indent=2) + "\n").encode()
    return output


def files(root):
    root = Path(root)
    required = [root / name for name in ("LICENSE", "README.md", "CHANGELOG.md", "CONTRIBUTING.md", "AGENTS.md",
                                         "mise.toml", "package.json", "bun.lock", ".gitignore", ".prettierignore", ".prettierrc.json")]
    required.append(root / "dist/accessibility-contrast.json")
    required += [root / "dist" / f"{mode}.{suffix}" for mode in MODES for suffix in SUFFIXES]
    for folder in ("dist/svelte", "dist/storybook", "docs", "assets", "examples", "tokens", "src", "scripts", "tests", "patches", "reference/svelte"):
        path = root / folder
        if not path.is_dir():
            raise ValueError(f"missing candidate directory: {path}")
        for directory, names, filenames in os.walk(path, followlinks=False):
            names[:] = [name for name in names if name not in ("node_modules", "__pycache__", ".verification", ".svelte-kit", ".svelte-check") and not (Path(directory) / name).is_symlink()]
            required.extend(Path(directory) / name for name in filenames if not name.endswith((".pyc", ".pyo")) and not (Path(directory) / name).is_symlink())
    output = {}
    for path in sorted(required):
        if not path.is_file() or path.is_symlink():
            raise ValueError(f"missing candidate file: {path}")
        rel = path.relative_to(root).as_posix()
        if rel.startswith("dist/"):
            rel = rel.removeprefix("dist/")
        if rel in output:
            raise ValueError(f"duplicate candidate path: {rel}")
        output[rel] = path.read_bytes()
    return output


def bundle(root):
    contents = files(root)
    contents.update(dependency_notices(root))
    svelte = json.loads(contents["svelte/package.json"])
    entries = {name: sha256(content).hexdigest() for name, content in sorted(contents.items())}
    manifest = {"status": "candidate", "mvpVersion": VERSION, "tokenContract": metadata()["contractVersion"],
                "svelteVersion": svelte["version"], "sha256": entries}
    contents["manifest.json"] = (json.dumps(manifest, ensure_ascii=False, indent=2) + "\n").encode()
    target = BytesIO()
    with ZipFile(target, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for name, content in sorted(contents.items()):
            info = ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.compress_type = ZIP_DEFLATED
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content, compresslevel=9)
    return target.getvalue()


if __name__ == "__main__":
    target = ROOT / "dist/release" / f"fjord-mvp-{VERSION}-candidate.zip"
    write_binary_output(target, bundle(ROOT), [ROOT / name for name in ("tokens", "src", "docs", "assets", "tests", "reference", "scripts", ".git")])
    print(f"Prepared local candidate: {target}")
