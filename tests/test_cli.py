import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]


class FoundationCliTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fjord-cli-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        for directory in ("tokens", "assets/fonts", "assets/icons"):
            shutil.copytree(ROOT / directory, self.root / directory)

    def cli(self, command, *arguments):
        return subprocess.run([sys.executable, "-m", "fjord_tokens", command,
                               "--root", str(self.root), *map(str, arguments)],
                              cwd=self.root, capture_output=True, text=True)

    def test_validate_checks_assets_without_documentation_or_implementations(self):
        result = self.cli("validate")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("430 contrast pairs", result.stdout)
        self.assertIn("9 font faces and 16 icons with pinned licences", result.stdout)
        self.assertFalse((self.root / "dist").exists())

    def test_validate_reports_corrupt_assets_and_licences_without_tracebacks(self):
        for relative in ("assets/fonts/cascadia-code/CascadiaCode.woff2",
                         "assets/fonts/fira-code/OFL.txt", "assets/icons/info.svg", "assets/icons/LICENSE"):
            with self.subTest(asset=relative):
                path = self.root / relative
                original = path.read_bytes()
                path.write_bytes(original + b"changed")
                try:
                    result = self.cli("validate")
                    self.assertEqual(result.returncode, 1)
                    self.assertIn("ERROR", result.stderr)
                    self.assertNotIn("Traceback", result.stderr)
                finally:
                    path.write_bytes(original)

    def test_validate_rejects_malformed_asset_manifest_shapes(self):
        for relative in ("assets/fonts/manifest.json", "assets/fonts/inspection.json", "assets/icons/manifest.json"):
            with self.subTest(manifest=relative):
                path = self.root / relative
                original = path.read_text()
                path.write_text(json.dumps([]))
                try:
                    result = self.cli("validate")
                    self.assertEqual(result.returncode, 1)
                    self.assertNotIn("Traceback", result.stderr)
                finally:
                    path.write_text(original)

    def test_token_build_and_export_do_not_require_asset_files(self):
        shutil.rmtree(self.root / "assets")
        for command, arguments in (("build", []), ("export", ["--format", "resolved"]),
                                   ("export", ["--format", "css"]), ("export", ["--format", "dtcg"])):
            with self.subTest(command=command, arguments=arguments):
                result = self.cli(command, *arguments)
                self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.cli("validate").returncode, 1)

    def test_build_and_export_cannot_replace_source_assets_through_symlinks(self):
        target = self.root / "assets/icons/info.svg"
        original = target.read_bytes()
        output = self.root / "output.json"
        output.symlink_to(target)
        for command, arguments in (("build", []), ("export", ["--format", "resolved"])):
            with self.subTest(command=command):
                result = self.cli(command, *arguments, "--output", output)
                self.assertEqual(result.returncode, 1)
                self.assertIn("must not replace token sources", result.stderr)
                self.assertEqual(target.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
