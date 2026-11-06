from hashlib import sha256
from io import BytesIO
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("package_candidate", ROOT / "scripts/package_candidate.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class CandidateTests(unittest.TestCase):
    def test_archive_is_reproducible_and_hashes_every_entry(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ("LICENSE", "README.md", "CHANGELOG.md", "CONTRIBUTING.md", "AGENTS.md", "mise.toml", "package.json", "bun.lock", ".gitignore", ".prettierignore", ".prettierrc.json", ".github/workflows/validate.yml"):
                (root / name).parent.mkdir(parents=True, exist_ok=True)
                (root / name).write_text(name)
            for directory in ("dist/svelte", "dist/storybook", "docs", "licenses", "assets", "examples", "tokens", "src", "scripts", "tests", "patches", "reference/svelte", "adapters"):
                (root / directory).mkdir(parents=True)
                (root / directory / "sample.txt").write_text(directory)
            (root / "dist/svelte/package.json").write_text('{"version":"0.8.0"}')
            (root / "dist/accessibility-contrast.json").write_text('{}')
            zed = root / "adapters/zed"
            zed.mkdir()
            adapter_files = ("adapter.json", "generate.py", "schema/themes-v0.2.0.json", "schema/LICENSE-GPL")
            for name in adapter_files:
                (zed / name).parent.mkdir(parents=True, exist_ok=True)
                (zed / name).write_text(name)
            vscode = root / "adapters/vscode"
            vscode.mkdir()
            vscode_files = ("adapter.json", "generate.py", "package.json", "README.md", "LICENSE")
            for name in vscode_files:
                (vscode / name).write_text(name)
            (root / "scripts/package_vscode.py").write_text("VSIX packager")
            dependency = root / "node_modules/.bun/example@1.0.0/node_modules/example"
            dependency.mkdir(parents=True)
            (dependency / "package.json").write_text('{"name":"example","version":"1.0.0","license":"MIT"}')
            (dependency / "LICENSE").write_text("Example licence")
            (dependency / "ThirdPartyNoticeText.txt").write_text("Third-party notice")
            (dependency / "vendor").mkdir()
            (dependency / "vendor/LICENSE.txt").write_text("Nested licence")
            for mode in module.MODES:
                for suffix in module.SUFFIXES:
                    (root / "dist" / f"{mode}.{suffix}").write_text(f"{mode}.{suffix}")
            ignored = root / "reference/svelte/node_modules"
            ignored.mkdir()
            (ignored / "secret.txt").write_text("excluded")
            first = module.bundle(root)
            self.assertEqual(first, module.bundle(root))
            with ZipFile(BytesIO(first)) as archive:
                manifest = json.loads(archive.read("manifest.json"))
                self.assertEqual(manifest["status"], "candidate")
                self.assertEqual(manifest["svelteVersion"], "0.8.0")
                self.assertIn(".github/workflows/validate.yml", archive.namelist())
                for name in adapter_files:
                    self.assertEqual(archive.read("adapters/zed/" + name), name.encode())
                for name in vscode_files:
                    self.assertEqual(archive.read("adapters/vscode/" + name), name.encode())
                self.assertEqual(archive.read("scripts/package_vscode.py"), b"VSIX packager")
                self.assertEqual(set(manifest["sha256"]), set(archive.namelist()) - {"manifest.json"})
                self.assertNotIn("reference/svelte/node_modules/secret.txt", archive.namelist())
                notices = json.loads(archive.read("third-party-notices/index.json"))["packages"]
                self.assertEqual(len(notices), 1)
                record = next(iter(notices.values()))
                self.assertEqual(record["name"], "example")
                self.assertEqual(archive.read(record["notices"][0]), b"Example licence")
                self.assertEqual(len(record["notices"]), 3)
                self.assertEqual({archive.read(name) for name in record["notices"]},
                                 {b"Example licence", b"Third-party notice", b"Nested licence"})
                for name, digest in manifest["sha256"].items():
                    self.assertEqual(sha256(archive.read(name)).hexdigest(), digest)
            (root / "dist/night.wezterm.toml").unlink()
            with self.assertRaisesRegex(ValueError, "missing candidate file"):
                module.bundle(root)


if __name__ == "__main__":
    unittest.main()
