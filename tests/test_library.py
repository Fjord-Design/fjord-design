from hashlib import sha256
from io import BytesIO
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import package_library as LIBRARY
from check_candidate import verify


class LibraryTests(unittest.TestCase):
    def test_complete_archive_is_reproducible_and_keeps_sources_and_native_outputs(self):
        source = {"svelte/package.json": b'{"version":"1.0.0-rc.1"}',
                  "reference/svelte/package.json": b'{"version":"1.0.0-rc.1"}',
                  "tokens/contract.json": b'{"version":"0.3.0"}',
                  "docs/support-matrix.md": b"Verified target limits",
                  "adapters/wezterm/README.md": b"source installation guide"}
        with tempfile.TemporaryDirectory() as tmp, patch.object(LIBRARY, "files", return_value=source), patch.object(LIBRARY, "dependency_notices", return_value={}):
            root = Path(tmp)
            for name in (*[f"adapters/{target}/report.json" for target in LIBRARY.PLATFORMS], "adapters/wezterm/README.md", "variants/work.css", "variants/gaming.css", "release/fjord-themes-0.1.0.vsix"):
                file = root / "dist" / name
                file.parent.mkdir(parents=True, exist_ok=True)
                file.write_text(name)
            first = LIBRARY.bundle(root)
            self.assertEqual(first, LIBRARY.bundle(root))
            with ZipFile(BytesIO(first)) as archive:
                manifest = verify(archive)
                self.assertEqual(manifest["componentCount"], 66)
                self.assertEqual(manifest["libraryVersion"], "1.0.0-rc.1")
                self.assertEqual(archive.read("adapters/wezterm/README.md"), b"source installation guide")
                self.assertEqual(archive.read("platforms/wezterm/README.md"), b"adapters/wezterm/README.md")
                for name, digest in manifest["sha256"].items():
                    self.assertEqual(sha256(archive.read(name)).hexdigest(), digest)
            (root / "dist/adapters/kde/report.json").unlink()
            with self.assertRaisesRegex(ValueError, "missing platform report"):
                LIBRARY.bundle(root)
