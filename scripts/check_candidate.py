"""Verify the local candidate archive and build its extracted source in isolation."""

from hashlib import sha256
import json
import os
from pathlib import Path, PurePosixPath
import stat
import subprocess
import sys
import tempfile
from zipfile import BadZipFile, ZipFile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / "dist/release/fjord-mvp-0.1.0-candidate.zip"


def read_object(archive, name):
    value = json.loads(archive.read(name))
    if not isinstance(value, dict):
        raise ValueError(f"candidate JSON must be an object: {name}")
    return value


def verify(archive):
    """Check exact membership, safe file paths, hashes and version consistency."""
    names = archive.namelist()
    if len(names) != len(set(names)):
        raise ValueError("candidate contains duplicate archive paths")
    for info in archive.infolist():
        path = PurePosixPath(info.filename)
        if (path.is_absolute() or ".." in path.parts or "\\" in info.filename
                or path.as_posix() != info.filename or info.is_dir()
                or stat.S_ISLNK(info.external_attr >> 16)):
            raise ValueError(f"candidate contains an unsafe file path: {info.filename}")
    if "manifest.json" not in names:
        raise ValueError("candidate is missing manifest.json")
    manifest = read_object(archive, "manifest.json")
    hashes = manifest.get("sha256")
    if not isinstance(hashes, dict) or set(hashes) != set(names) - {"manifest.json"}:
        raise ValueError("candidate manifest does not match archive membership")
    for name, digest in hashes.items():
        if sha256(archive.read(name)).hexdigest() != digest:
            raise ValueError(f"candidate checksum mismatch: {name}")
    if manifest.get("status") != "candidate":
        raise ValueError("expected an unaccepted candidate")
    for field, name in (("svelteVersion", "svelte/package.json"),
                        ("svelteVersion", "reference/svelte/package.json"),
                        ("tokenContract", "tokens/contract.json")):
        if name not in hashes:
            raise ValueError(f"candidate is missing version source: {name}")
        version = read_object(archive, name).get("version")
        if not isinstance(version, str) or manifest.get(field) != version:
            raise ValueError(f"candidate version mismatch: {name}")
    return manifest


def main(archive_path=ARCHIVE):
    with ZipFile(archive_path) as archive:
        manifest = verify(archive)
        print(f"PASS candidate integrity: {len(manifest['sha256'])} files", flush=True)
        with tempfile.TemporaryDirectory(prefix="fjord-candidate-") as temporary:
            directory = Path(temporary)
            archive.extractall(directory)
            # Build from source, without reusing the archive's prebuilt outputs.
            # The archive intentionally stores these outside dist/.
            env = {**os.environ, "MISE_TRUSTED_CONFIG_PATHS": str(directory),
                   "PYTHONPATH": str(directory / "src")}
            for task in ("setup", "ci"):
                subprocess.run(["mise", "run", task], cwd=directory, env=env, check=True)
    print("PASS extracted candidate installs locked dependencies and rebuilds from its own sources")


if __name__ == "__main__":
    try:
        import argparse
        parser = argparse.ArgumentParser(description=__doc__)
        parser.add_argument("archive", nargs="?", type=Path, default=ARCHIVE)
        main(parser.parse_args().archive)
    except (OSError, ValueError, BadZipFile, subprocess.CalledProcessError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        sys.exit(1)
