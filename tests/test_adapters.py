import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from fjord_tokens.adapters import load_adapter, map_adapter, write_adapter_files
from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import TokenError
from fjord_tokens.export import wezterm_document
from fjord_tokens.project import Project, validate_project


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "adapters/wezterm/adapter.json"
SPEC = importlib.util.spec_from_file_location("wezterm_adapter", ROOT / "adapters/wezterm/generate.py")
WEZTERM = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(WEZTERM)


class AdapterTests(unittest.TestCase):
    def test_reference_schemes_match_existing_export_for_both_identities(self):
        project = Project(ROOT)
        for theme in (None, ROOT / "examples/themes/fern.tokens.json"):
            with self.subTest(theme=theme):
                files = WEZTERM.bundle(ROOT, theme)
                self.assertEqual(files, WEZTERM.bundle(ROOT, theme))
                for mode in THEMES:
                    self.assertEqual(files[f"{mode}.toml"], wezterm_document(project.compose(mode, theme)))
                report = json.loads(files["report.json"])
                self.assertEqual(report["target"]["application"], "WezTerm")
                night = report["modes"]["night"]
                self.assertEqual(night["mapped"]["cursor_bg"]["role"], "semantic.color.accent")
                self.assertIn("semantic.color.syn-comment", night["unmapped"])
                self.assertIn("source", night["mapped"]["cursor_bg"])
                self.assertEqual(night["unsupported"], {})

    def test_manifest_structure_rejects_typos_types_and_missing_mappings(self):
        original = json.loads(MANIFEST.read_text())
        changes = [
            lambda value: value.update(schemaVersion=True),
            lambda value: value.update(schemaVersion=2),
            lambda value: value.update(unexpected="ignored"),
            lambda value: value.update(id="WezTerm"),
            lambda value: value["target"].update(version=None),
            lambda value: value.update(modes=["night", "night"]),
            lambda value: value.update(modes=["not-a-mode"]),
            lambda value: value.update(mapping=[]),
            lambda value: value["mapping"].update(background=32),
            lambda value: value.update(requiredRoles=["semantic.color.syn-comment"]),
            lambda value: value.update(requiredRoles=["semantic.color.ink", "semantic.color.ink"]),
            lambda value: value["unsupported"].update({"semantic.color.ink": "unsupported"}),
            lambda value: value["fallbacks"].update(fonts=""),
        ]
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "adapter.json"
            for index, change in enumerate(changes):
                with self.subTest(index=index):
                    value = json.loads(json.dumps(original))
                    change(value)
                    manifest.write_text(json.dumps(value))
                    with self.assertRaises(TokenError):
                        load_adapter(ROOT, manifest)

    def test_unknown_mapped_and_unsupported_roles_fail_before_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "adapter.json"
            for section in ("mapping", "unsupported"):
                value = json.loads(MANIFEST.read_text())
                if section == "mapping":
                    value[section]["unrecognized"] = "semantic.color.nonexistent"
                else:
                    value[section]["semantic.color.nonexistent"] = "No target equivalent"
                manifest.write_text(json.dumps(value))
                with self.subTest(section=section), self.assertRaisesRegex(TokenError, "unknown semantic roles"):
                    map_adapter(ROOT, manifest)

    def test_semantic_values_and_explicit_unsupported_roles_are_portable(self):
        value = json.loads(MANIFEST.read_text())
        value["mapping"]["font"] = "semantic.font.mono"
        value["unsupported"]["semantic.color.syn-comment"] = "Terminal applications choose syntax colours."
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "adapter.json"
            manifest.write_text(json.dumps(value))
            with patch("fjord_tokens.adapters.validate_project", wraps=validate_project) as validate:
                modes, report = map_adapter(ROOT, manifest)
            validate.assert_called_once_with(ROOT)
        expected = Project(ROOT).compose("night").values["semantic.font.mono"]
        self.assertEqual(modes["night"]["font"], expected)
        self.assertEqual(report["modes"]["night"]["unsupported"], value["unsupported"])
        self.assertNotIn("semantic.color.syn-comment", report["modes"]["night"]["unmapped"])

    def test_ordered_overrides_reach_the_mapped_output(self):
        theme = ROOT / "examples/themes/fern.tokens.json"
        modes, report = map_adapter(ROOT, MANIFEST, overrides=[theme])
        expected = WEZTERM.bundle(ROOT, theme)
        self.assertEqual(WEZTERM.scheme("night", modes["night"]), expected["night.toml"])
        self.assertEqual(report["modes"]["night"]["mapped"]["cursor_bg"]["source"],
                         "override:1:fern.tokens.json")

    def test_all_destination_guards_run_before_any_file_is_written(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "output"
            input_file = Path(tmp) / "theme.json"
            input_file.write_text("keep me")
            for bad in ("../escape.txt", str(Path(tmp) / "absolute.txt")):
                with self.subTest(bad=bad), self.assertRaises(TokenError):
                    write_adapter_files(ROOT, output, {"valid.txt": "safe", bad: "unsafe"})
                self.assertFalse(output.exists())
            with self.assertRaisesRegex(TokenError, "must not replace"):
                write_adapter_files(ROOT, input_file.parent, {input_file.name: "overwrite"}, [input_file])
            self.assertEqual(input_file.read_text(), "keep me")
            output.mkdir()
            (output / "source-link").symlink_to(ROOT / "tokens", target_is_directory=True)
            with self.assertRaises(TokenError):
                write_adapter_files(ROOT, output, {"source-link/base.tokens.json": "overwrite"})
            write_adapter_files(ROOT, output, {"night.toml": "safe"})
            self.assertEqual((output / "night.toml").read_text(), "safe")

    def test_reference_cli_reports_failures_without_overwriting_sources(self):
        command = [sys.executable, str(ROOT / "adapters/wezterm/generate.py")]
        result = subprocess.run(command + ["--help"], cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--output-dir", result.stdout)
        with tempfile.TemporaryDirectory() as tmp:
            result = subprocess.run(command + ["--output-dir", tmp], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((Path(tmp) / "report.json").is_file())
        result = subprocess.run(command + ["--output-dir", str(ROOT / "adapters/wezterm")],
                                cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn("must not replace", result.stderr)
        self.assertFalse((ROOT / "adapters/wezterm/night.toml").exists())

    def test_reference_serializer_rejects_wrong_targets_and_non_colour_values(self):
        modes, _ = map_adapter(ROOT, MANIFEST)
        for values in (dict(modes["night"], cursor_bg="not a colour"),
                       {key: value for key, value in modes["night"].items() if key != "background"},
                       dict(modes["night"], extra="#ffffff")):
            with self.subTest(values=values), self.assertRaises(TokenError):
                WEZTERM.scheme("night", values)


if __name__ == "__main__":
    unittest.main()
