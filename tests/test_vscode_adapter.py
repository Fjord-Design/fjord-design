from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from fjord_tokens.adapters import map_adapter
from fjord_tokens.contract import ANSI, THEMES
from fjord_tokens.dtcg import TokenError, hex_color
from fjord_tokens.project import Project, check_contrast, metadata


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("vscode_adapter", ROOT / "adapters/vscode/generate.py")
vscode = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(vscode)


class VSCodeAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.files = vscode.build_bundle(ROOT)
        cls.report = json.loads(cls.files["report.json"])
        cls.mapped, cls.mapping_report = map_adapter(ROOT, ROOT / "adapters/vscode/adapter.json")

    def test_extension_declares_three_typed_themes_without_executable_content(self):
        package = json.loads(self.files["package.json"])
        vscode.validate_package(package, self.files)
        self.assertTrue(all(isinstance(value, str) for value in self.files.values()))
        for mode, contribution in zip(THEMES, package["contributes"]["themes"]):
            document = json.loads(self.files[contribution["path"][2:]])
            self.assertEqual(document["name"], "Fjord " + mode.title())
            self.assertEqual(document["type"], "light" if mode == "day" else "dark")
            self.assertTrue(document["semanticHighlighting"])
            self.assertEqual(self.report["modes"][mode]["unmapped"], [])
        invalid = deepcopy(package)
        invalid["main"] = "extension.js"
        with self.assertRaisesRegex(TokenError, "executable"):
            vscode.validate_package(invalid, self.files)

    def test_modes_use_shared_roles_and_all_sixteen_ansi_slots(self):
        for mode in THEMES:
            values = Project(ROOT).compose(mode).values
            document = json.loads(self.files[f"themes/fjord-{mode}-color-theme.json"])
            colors = document["colors"]
            for field, role in (("editor.background", "bg-base"), ("editor.foreground", "ink"),
                                ("terminal.selectionBackground", "selection"), ("terminal.selectionForeground", "ink")):
                self.assertEqual(colors[field], hex_color(values["semantic.color." + role]))
            self.assertNotIn("editor.selectionForeground", colors)
            if mode != "day":
                self.assertEqual(colors["editor.selectionBackground"], hex_color(values["semantic.color.selection"]))
            for name in ANSI:
                field = "terminal.ansi" + "".join(part.title() for part in name.removeprefix("ansi-").split("-"))
                self.assertEqual(colors[field], hex_color(values["semantic.color." + name]))
            self.assertEqual(document["semanticTokenColors"]["comment"]["foreground"],
                             hex_color(values["semantic.color.syn-comment"]))
        aurora = json.loads(self.files["themes/fjord-aurora-color-theme.json"])
        self.assertEqual(aurora["colors"]["editor.selectionBackground"], "#093b3b")

    def test_normal_editor_selection_checks_retained_syntax_instead_of_ignored_foreground(self):
        self.assertFalse(self.report["capabilities"]["editorSelectionForeground"]["standardThemes"])
        for mode in THEMES:
            checks = [item for item in self.report["contrast"][mode]
                      if item["background"] in ("colors.editor.selectionBackground", "colors.editor.inactiveSelectionBackground")]
            self.assertTrue(any(item["foreground"] == "semantic.comment" for item in checks))
            self.assertTrue(any(item["foreground"] == "tm.property" for item in checks))
            self.assertFalse(any(item["foreground"] == "colors.editor.selectionForeground" for item in checks))
        day = json.loads(self.files["themes/fjord-day-color-theme.json"])["colors"]
        source = self.mapped["day"]["colors.editor.selectionBackground"]
        comment = self.mapped["day"]["tm.comment"]
        self.assertLess(min(vscode.pair(comment, source, source)[:2]), 4.5)
        self.assertGreaterEqual(min(vscode.pair(comment, day["editor.selectionBackground"], day["editor.background"])[:2]), 4.5)

    def test_json_property_scope_is_more_specific_than_generic_type_and_string(self):
        scopes = self.report["contractEvidence"]["scopeExamples"]["jsonProperty"]
        rules = json.loads(self.files["themes/fjord-night-color-theme.json"])["tokenColors"]
        matches = [(len(selector.split(".")), rule["name"])
                   for rule in rules for selector in rule["scope"]
                   if any(scope == selector or scope.startswith(selector + ".") for scope in scopes)]
        self.assertEqual(max(matches)[1], "Fjord property")
        self.assertIn("support.type.property-name", vscode.SCOPES["property"])

    def test_composited_contrast_covers_selection_find_diff_and_terminal(self):
        for mode in THEMES:
            checks = self.report["contrast"][mode]
            backgrounds = {item["background"] for item in checks}
            self.assertTrue({"colors.editor.selectionBackground", "colors.editor.inactiveSelectionBackground",
                             "colors.editor.findMatchBackground", "colors.editor.findMatchHighlightBackground",
                             "colors.terminal.selectionBackground", "colors.terminal.inactiveSelectionBackground",
                             "colors.terminal.findMatchBackground", "colors.terminal.findMatchHighlightBackground",
                             "colors.diffEditor.insertedTextBackground", "colors.diffEditor.removedTextBackground"}
                            .issubset(backgrounds))
            for item in checks:
                underlay = {"colorSpace": "srgb", "components": item["underlayComponents"], "alpha": 1}
                unrounded, rounded, background = vscode.pair(item["foregroundHex"], item["backgroundHex"], underlay)
                self.assertEqual(item["unroundedRatio"], unrounded)
                self.assertEqual(item["hexRatio"], rounded)
                self.assertEqual(item["compositedBackgroundHex"], background)
                self.assertEqual(item["ratio"], min(unrounded, rounded))
                self.assertGreaterEqual(item["ratio"], item["minimum"])
                self.assertTrue(item["pass"])
            terminal_find = [item for item in checks if item["background"] == "colors.terminal.findMatchBackground"]
            self.assertTrue(any(item["foreground"] == "colors.terminal.ansiBrightBlack" for item in terminal_find))
            self.assertFalse(any("LineNumber" in item["foreground"] for item in terminal_find))

    def test_diff_layers_keep_full_precision_until_final_hex_quantization(self):
        check = next(item for item in self.report["contrast"]["night"]
                     if item["foreground"] == "semantic.class"
                     and item["background"] == "colors.diffEditor.insertedTextBackground")
        self.assertEqual(check["foregroundHex"], "#92c1e3")
        # Independent sRGB compositing of base #111920, line #1d2a1d30 and
        # character fill #1d2a1d60 yields this ratio before any HEX rounding.
        self.assertAlmostEqual(check["unroundedRatio"], 8.578806519, places=8)

    def test_alpha_substitutions_retain_original_mapping_and_actual_emitted_value(self):
        for mode in THEMES:
            colors = json.loads(self.files[f"themes/fjord-{mode}-color-theme.json"])["colors"]
            for substitution in self.report["substitutions"][mode]:
                target = substitution["target"]
                binding = self.report["modes"][mode]["mapped"][target]
                self.assertEqual(binding["value"], substitution["source"])
                self.assertEqual(binding["emittedValue"], substitution["emitted"])
                self.assertEqual(colors[target.removeprefix("colors.")], substitution["emitted"])
            self.assertLess(int(colors["terminal.findMatchBackground"][-2:], 16), 255)
            self.assertEqual(colors["terminal.findMatchBorder"], self.mapped[mode]["colors.terminal.findMatchBorder"])

    def test_unknown_targets_non_colour_values_and_unverified_target_versions_fail(self):
        cases = [
            ("colors.editor.forground", "#ffffff", "unknown VS Code"),
            ("tm.not-a-role", "#ffffff", "unknown TextMate"),
            ("semantic.class.invalid", "#ffffff", "unsupported semantic"),
            ("colors.editor.foreground", {"value": 14, "unit": "px"}, "opaque semantic colour"),
        ]
        for target, value, message in cases:
            mapped = deepcopy(self.mapped)
            mapped["night"][target] = value
            with self.subTest(target=target), patch.object(vscode, "map_adapter", return_value=(mapped, deepcopy(self.mapping_report))):
                with self.assertRaisesRegex(TokenError, message):
                    vscode.build_bundle(ROOT)
        report = deepcopy(self.mapping_report)
        report["target"]["version"] = "1.0.0"
        with patch.object(vscode, "map_adapter", return_value=(deepcopy(self.mapped), report)):
            with self.assertRaisesRegex(TokenError, "pinned target"):
                vscode.build_bundle(ROOT)
        mapped = deepcopy(self.mapped)
        del mapped["night"]["colors.editor.findMatchForeground"]
        with patch.object(vscode, "map_adapter", return_value=(mapped, deepcopy(self.mapping_report))):
            with self.assertRaisesRegex(TokenError, "missing required VS Code target mappings"):
                vscode.build_bundle(ROOT)

    def test_source_valid_override_can_fail_retained_diff_text_before_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            override = Path(tmp) / "strong-diff.tokens.json"
            override.write_text(json.dumps({
                "$extensions": {"org.xerrion.fjord": metadata()},
                "semantic": {"color": {
                    name: {"$type": "color", "$value": "{semantic.color." + role + "}",
                           "$description": "Source-valid ink text differs from retained syntax."}
                    for name, role in (("diff-add-bg", "search-match-current"), ("git-added", "ink"))
                }},
            }), encoding="utf-8")
            for mode in THEMES:
                check_contrast(Project(ROOT).compose(mode, overrides=[override]))
            with self.assertRaisesRegex(TokenError, r"diffEditor\.insertedTextBackground:.*below 4\.5:1"):
                vscode.build_bundle(ROOT, overrides=[override])

    def test_fern_reuses_components_and_changes_native_accent_mapping(self):
        fern = ROOT / "examples/themes/fern.tokens.json"
        files = vscode.build_bundle(ROOT, theme=fern)
        for mode in THEMES:
            values = Project(ROOT).compose(mode, fern).values
            document = json.loads(files[f"themes/fjord-{mode}-color-theme.json"])
            self.assertEqual(document["colors"]["button.background"], hex_color(values["semantic.color.accent"]))
        self.assertNotEqual(files["themes/fjord-night-color-theme.json"], self.files["themes/fjord-night-color-theme.json"])

    def test_bundle_is_deterministic_and_cli_preserves_sources_inputs_and_settings(self):
        self.assertEqual(vscode.build_bundle(ROOT), self.files)
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            (directory / "project").symlink_to(ROOT, target_is_directory=True)
            output = directory / "generated"
            command = [sys.executable, str(ROOT / "adapters/vscode/generate.py"), "--root", "project", "--output-dir", str(output)]
            result = subprocess.run(command, cwd=directory, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((output / "themes/fjord-aurora-color-theme.json").read_text(), self.files["themes/fjord-aurora-color-theme.json"])
            source = ROOT / "adapters/vscode/package.json"
            before = source.read_bytes()
            result = subprocess.run(command[:-1] + [str(ROOT / "adapters/vscode")], cwd=directory,
                                    text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must not replace", result.stderr)
            self.assertEqual(source.read_bytes(), before)
            collision = directory / "collision"
            collision.mkdir()
            input_path = collision / "report.json"
            input_path.write_bytes((ROOT / "examples/overrides/compact-spacing.tokens.json").read_bytes())
            before = input_path.read_bytes()
            result = subprocess.run(command[:-1] + [str(collision), "--override", str(input_path)], cwd=directory,
                                    text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(input_path.read_bytes(), before)
            self.assertFalse((collision / "themes").exists())
            self.assertFalse((output / "settings.json").exists())


if __name__ == "__main__":
    unittest.main()
