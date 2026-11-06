from copy import deepcopy
from io import BytesIO
import importlib.util
import json
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest
from xml.etree import ElementTree as XML
from zipfile import ZipFile

from fjord_tokens.dtcg import TokenError


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("package_vscode", ROOT / "scripts/package_vscode.py")
packager = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(packager)


def example_files():
    themes = [{"label": "Fjord " + mode.title(), "uiTheme": "vs" if mode == "day" else "vs-dark",
               "path": f"./themes/fjord-{mode}-color-theme.json"} for mode in ("night", "day", "aurora")]
    manifest = {"name": "fjord-themes", "publisher": "fjord", "version": "0.1.0",
                "displayName": "Fjord", "description": "Calm Nordic themes", "engines": {"vscode": "^1.100.0"},
                "categories": ["Themes"], "contributes": {"themes": themes}}
    files = {"package.json": json.dumps(manifest), "README.md": "# Fjord\n", "LICENSE": "Fjord MIT licence\n",
             "report.json": "private generation evidence", "generate.py": "must not be packaged",
             "unlisted.txt": "must not be packaged"}
    files.update({theme["path"][2:]: json.dumps({"name": theme["label"], "colors": {}, "tokenColors": []})
                  for theme in themes})
    return files


def replace_manifest(files, change):
    result = deepcopy(files)
    manifest = json.loads(result["package.json"])
    change(manifest)
    result["package.json"] = json.dumps(manifest)
    return result


def fixture_generator(root, files):
    directory = root / "adapters/vscode"
    directory.mkdir(parents=True)
    (directory / "generate.py").write_text(
        "def build_bundle(root, theme=None, overrides=()):\n    return " + repr(files) + "\n", encoding="utf-8")


class VSCodePackageTests(unittest.TestCase):
    def test_archive_identity_assets_and_deterministic_metadata(self):
        files = example_files()
        content = packager.package_files(files)
        self.assertEqual(content, packager.package_files(dict(reversed(list(files.items())))))
        with ZipFile(BytesIO(content)) as archive:
            expected = {"[Content_Types].xml", "extension.vsixmanifest", "extension/package.json",
                        "extension/README.md", "extension/LICENSE"}
            expected.update(f"extension/themes/fjord-{mode}-color-theme.json" for mode in ("night", "day", "aurora"))
            self.assertEqual(set(archive.namelist()), expected)
            self.assertEqual(archive.namelist(), sorted(expected))
            self.assertFalse(any("report" in name or name.endswith((".py", ".js")) for name in archive.namelist()))
            for info in archive.infolist():
                self.assertEqual(info.date_time, (1980, 1, 1, 0, 0, 0))
                self.assertEqual(info.create_system, 3)
                self.assertEqual(stat.S_IMODE(info.external_attr >> 16), 0o644)
                self.assertTrue(stat.S_ISREG(info.external_attr >> 16))
            namespace = {"v": packager.VSIX_NAMESPACE}
            manifest = XML.fromstring(archive.read("extension.vsixmanifest"))
            identity = manifest.find("v:Metadata/v:Identity", namespace)
            self.assertEqual(identity.attrib, {"Language": "en-US", "Id": "fjord-themes", "Version": "0.1.0", "Publisher": "fjord"})
            properties = {item.attrib["Id"]: item.attrib["Value"]
                          for item in manifest.findall("v:Metadata/v:Properties/v:Property", namespace)}
            self.assertEqual(properties["Microsoft.VisualStudio.Code.Engine"], "^1.100.0")
            self.assertEqual(properties["Microsoft.VisualStudio.Code.ExecutesCode"], "false")
            assets = manifest.findall("v:Assets/v:Asset", namespace)
            self.assertEqual({item.attrib["Path"] for item in assets},
                             {"extension/package.json", "extension/README.md", "extension/LICENSE"})
            for item in assets:
                self.assertIn(item.attrib["Path"], archive.namelist())
            types = XML.fromstring(archive.read("[Content_Types].xml"))
            self.assertEqual(types.tag, "{" + packager.CONTENT_NAMESPACE + "}Types")
            content_namespace = {"c": packager.CONTENT_NAMESPACE}
            defaults = {item.attrib["Extension"]: item.attrib["ContentType"]
                        for item in types.findall("c:Default", content_namespace)}
            self.assertEqual(defaults, {"json": "application/json", "md": "text/markdown", "vsixmanifest": "text/xml"})
            self.assertEqual(types.find("c:Override", content_namespace).attrib,
                             {"PartName": "/extension/LICENSE", "ContentType": "text/plain"})
            self.assertEqual(archive.read("extension/package.json"), files["package.json"].encode())

    def test_metadata_is_xml_escaped_and_invalid_characters_fail(self):
        description = "Snow & frost <quiet> \"days\" 'nights'"
        files = replace_manifest(example_files(), lambda manifest: manifest.update(displayName=description, description=description))
        with ZipFile(BytesIO(packager.package_files(files))) as archive:
            manifest = XML.fromstring(archive.read("extension.vsixmanifest"))
            namespace = {"v": packager.VSIX_NAMESPACE}
            self.assertEqual(manifest.find("v:Metadata/v:DisplayName", namespace).text, description)
            self.assertEqual(manifest.find("v:Metadata/v:Description", namespace).text, description)
        for value in ("bad\x00metadata", "bad\ud800metadata", 42):
            files = replace_manifest(example_files(), lambda manifest: manifest.update(description=value))
            with self.subTest(value=repr(value)), self.assertRaises(TokenError):
                packager.package_files(files)

    def test_manifest_rejects_identity_code_or_missing_theme_data(self):
        changes = [lambda manifest: manifest.update(name="other"),
                   lambda manifest: manifest.update(publisher="other"),
                   lambda manifest: manifest.update(version="../outside"),
                   lambda manifest: manifest.update(engines={"vscode": "anything"}),
                   lambda manifest: manifest.update(main="extension.js"),
                   lambda manifest: manifest.update(activationEvents=["*"]),
                   lambda manifest: manifest["contributes"].update(commands=[]),
                   lambda manifest: manifest["contributes"]["themes"].pop(),
                   lambda manifest: manifest["contributes"]["themes"][0].update(uiTheme="unknown"),
                   lambda manifest: manifest["contributes"]["themes"][1].update(label="Fjord Night")]
        for index, change in enumerate(changes):
            with self.subTest(index=index), self.assertRaises(TokenError):
                packager.package_files(replace_manifest(example_files(), change))
        for missing in ("LICENSE", "README.md", "themes/fjord-day-color-theme.json"):
            files = example_files()
            del files[missing]
            with self.subTest(missing=missing), self.assertRaises(TokenError):
                packager.package_files(files)
        for name in ("package.json", "themes/fjord-day-color-theme.json"):
            files = example_files()
            files[name] = '{"duplicate":1,"duplicate":2}'
            with self.subTest(name=name), self.assertRaises(TokenError):
                packager.package_files(files)

    def test_contribution_paths_cannot_traverse_or_reference_executable_files(self):
        paths = ("../themes/fjord-night-color-theme.json", "/themes/fjord-night-color-theme.json",
                 "themes/../fjord-night-color-theme.json", "themes\\fjord-night-color-theme.json",
                 "themes//fjord-night-color-theme.json", "themes/fjord.js", "themes/nested/fjord.json")
        for path in paths:
            files = replace_manifest(example_files(), lambda manifest: manifest["contributes"]["themes"][0].update(path=path))
            with self.subTest(path=path), self.assertRaisesRegex(TokenError, "unsafe theme contribution"):
                packager.package_files(files)

    def test_bundle_uses_fresh_in_memory_generation_instead_of_stale_dist_files(self):
        files = example_files()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixture_generator(root, files)
            stale = root / "dist/adapters/vscode"
            stale.mkdir(parents=True)
            (stale / "package.json").write_text("stale invalid data")
            self.assertEqual(packager.bundle(root), packager.package_files(files))
            self.assertEqual((stale / "package.json").read_text(), "stale invalid data")

    def test_cli_writes_only_after_validation_and_preserves_sources(self):
        command = [sys.executable, str(ROOT / "scripts/package_vscode.py")]
        help_result = subprocess.run(command + ["--help"], cwd=ROOT, text=True, capture_output=True)
        self.assertEqual(help_result.returncode, 0, help_result.stderr)
        self.assertIn("--output", help_result.stdout)
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            root = directory / "source"
            fixture_generator(root, example_files())
            (root / "README.md").write_text("keep source")
            result = subprocess.run(command + ["--root", "source"], cwd=directory, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            output = root / "dist/release/fjord-themes-0.1.0.vsix"
            self.assertEqual(output.read_bytes(), packager.package_files(example_files()))
            for target in (root / "README.md", root / "adapters/vscode/README.md"):
                result = subprocess.run(command + ["--root", str(root), "--output", str(target)],
                                        cwd=directory, text=True, capture_output=True)
                self.assertEqual(result.returncode, 1)
                self.assertIn("must not replace", result.stderr)
            self.assertEqual((root / "README.md").read_text(), "keep source")
            invalid_files = replace_manifest(example_files(), lambda manifest: manifest.update(main="code.js"))
            (root / "adapters/vscode/generate.py").write_text(
                "def build_bundle(root, theme=None, overrides=()):\n    return " + repr(invalid_files) + "\n", encoding="utf-8")
            before = output.read_bytes()
            result = subprocess.run(command + ["--root", str(root), "--output", str(output)],
                                    cwd=directory, text=True, capture_output=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(output.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
