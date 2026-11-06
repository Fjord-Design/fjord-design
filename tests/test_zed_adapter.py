from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.contract import ANSI
from fjord_tokens.dtcg import TokenError, hex_color
from fjord_tokens.project import Project, check_contrast, metadata


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("zed_generator", ROOT / "adapters/zed/generate.py")
zed = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(zed)


class ZedAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.family, cls.report = zed.build(ROOT)

    def test_family_matches_schema_and_shared_roles(self):
        schema, _ = zed.schema_document(ROOT)
        zed.validate_family(self.family, schema)
        self.assertEqual([theme["name"] for theme in self.family["themes"]],
                         ["Fjord Night", "Fjord Day", "Fjord Aurora"])
        for mode, theme in zip(("night", "day", "aurora"), self.family["themes"]):
            values = Project(ROOT).compose(mode).values
            style = theme["style"]
            self.assertEqual(style["editor.background"], hex_color(values["semantic.color.bg-base"]))
            self.assertEqual(style["syntax"]["comment"]["color"], hex_color(values["semantic.color.syn-comment"]))
            self.assertEqual(style["syntax"]["comment"]["font_style"], "italic")
            for slot in ANSI:
                target = "terminal." + slot.replace("ansi-", "ansi.").replace("-", "_")
                role = "ink-subtle" if mode == "aurora" and slot == "ansi-bright-black" else slot
                self.assertEqual(style[target], hex_color(values["semantic.color." + role]))
            self.assertEqual(self.report["modes"][mode]["unmapped"], [])
            self.assertIn("semantic.color.search-match-current", self.report["modes"][mode]["unsupported"])

    def test_disabled_elements_keep_surface_backgrounds_and_disabled_foregrounds(self):
        for identity, theme in (("fjord", None), ("fern", ROOT / "examples/themes/fern.tokens.json")):
            family = self.family if theme is None else zed.build(ROOT, theme=theme)[0]
            for mode, editor_theme in zip(("night", "day", "aurora"), family["themes"]):
                with self.subTest(identity=identity, mode=mode):
                    values = Project(ROOT).compose(mode, theme).values
                    style = editor_theme["style"]
                    for foreground in ("text.disabled", "icon.disabled"):
                        self.assertEqual(style[foreground], hex_color(values["semantic.color.ink-disabled"]))
                    for background, role in (("element.disabled", "bg-surface"),
                                             ("ghost_element.disabled", "bg-sunken")):
                        self.assertEqual(style[background], hex_color(values["semantic.color." + role]))
                        for foreground in ("text.disabled", "icon.disabled"):
                            self.assertNotEqual(style[foreground], style[background])

    def test_highlight_fallback_preserves_foregrounds_and_measures_emitted_hex(self):
        for mode, theme in zip(("night", "day", "aurora"), self.family["themes"]):
            values = Project(ROOT).compose(mode).values
            for name, highlight in theme["style"]["syntax"].items():
                source = self.report["modes"][mode]["mapped"]["syntax." + name]["role"]
                self.assertEqual(highlight["color"], hex_color(values[source]))
            substitutions = self.report["substitutions"][mode]
            if mode != "aurora":
                self.assertEqual(len(substitutions), 5)
                self.assertTrue(any(item["target"] == "player.selection" for item in substitutions))
            for item in self.report["contrast"][mode]:
                self.assertEqual(item["ratio"], zed.ratio(item["foregroundHex"], item["backgroundHex"]))
                self.assertGreaterEqual(item["ratio"], item["minimum"])
                self.assertTrue(item["pass"])
            selection_checks = [item for item in self.report["contrast"][mode]
                                if item["background"] == "player.selection"]
            self.assertTrue(any(item["foreground"] == "terminal.ansi.bright_black" for item in selection_checks))
            self.assertTrue(any(item["foreground"] == "syntax.comment" for item in selection_checks))
            self.assertFalse(any("line_number" in item["foreground"] for item in selection_checks))
            for surface in ("created.background", "deleted.background", "modified.background"):
                diff_checks = [item for item in self.report["contrast"][mode] if item["background"] == surface]
                self.assertTrue(any(item["foreground"] == "editor.foreground" for item in diff_checks))
                self.assertTrue(any(item["foreground"] == "syntax.comment" for item in diff_checks))
                self.assertFalse(any("line_number" in item["foreground"] for item in diff_checks))

    def test_aurora_selection_retains_visible_source_fill_with_explicit_hint_fallback(self):
        theme = next(theme for theme in self.family["themes"] if theme["name"] == "Fjord Aurora")
        style = theme["style"]
        source = Project(ROOT).compose("aurora").values
        source_selection = hex_color(source["semantic.color.selection"])
        self.assertEqual(source_selection, "#093b3b")
        self.assertEqual(style["players"][0]["selection"], source_selection)
        self.assertGreaterEqual(zed.ratio(style["players"][0]["selection"], style["editor.background"]),
                                zed.ratio(source_selection, style["editor.background"]))
        visibility = self.report["visibility"]["aurora"]
        self.assertEqual(visibility["previousSelectionHex"], "#031111")
        self.assertEqual(visibility["emittedSelectionHex"], source_selection)
        self.assertGreater(visibility["emittedSelectionBaseRatio"], visibility["previousSelectionBaseRatio"])
        substitutions = [item for item in self.report["substitutions"]["aurora"]
                         if "fallbackSemanticRole" in item]
        self.assertEqual(len(substitutions), 10)
        for item in substitutions:
            self.assertEqual(item["source"], "#6e7c8b")
            self.assertEqual(item["emitted"], "#92a1ae")
            self.assertEqual(item["fallbackSemanticRole"], "semantic.color.ink-subtle")
            self.assertEqual(style[item["target"]], item["emitted"])
            mapping = self.report["modes"]["aurora"]["mapped"][item["target"]]
            self.assertEqual(mapping["value"], item["source"])
            self.assertEqual(mapping["emittedValue"], item["emitted"])
            self.assertEqual(mapping["role"], item["sourceRole"])

    def test_original_highlights_reproduce_the_retained_text_failure(self):
        for mode in ("night", "day", "aurora"):
            values = Project(ROOT).compose(mode).values
            source = hex_color(values["semantic.color.selection"])
            retained = hex_color(values["semantic.color.ansi-bright-black"])
            self.assertLess(zed.ratio(retained, source), 4.5)

    def test_unsafe_highlight_fails_instead_of_hiding_text(self):
        with self.assertRaisesRegex(TokenError, "cannot retain"):
            zed.safe_highlight("#777777", {"black": "#000000", "white": "#ffffff", "mid": "#777777"}, "dark")

    def test_schema_rejects_unknown_keys_types_and_invalid_colors(self):
        schema, _ = zed.schema_document(ROOT)
        for field, value in (("editor.forground", "#ffffff"), ("editor.foreground", 42),
                             ("editor.foreground", "not-a-color")):
            bad = deepcopy(self.family)
            bad["themes"][0]["style"][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(TokenError):
                zed.validate_family(bad, schema)
        bad = deepcopy(self.family)
        bad["themes"][0]["style"]["players"][0]["selection_border"] = "#ffffff"
        with self.assertRaisesRegex(TokenError, "unknown Zed schema field"):
            zed.validate_family(bad, schema)

    def test_identity_override_changes_adapter_without_component_edits(self):
        fern = ROOT / "examples/themes/fern.tokens.json"
        family, _ = zed.build(ROOT, theme=fern)
        self.assertNotEqual(family["themes"][0]["style"]["text.accent"],
                            self.family["themes"][0]["style"]["text.accent"])
        for mode, theme in zip(("night", "day", "aurora"), family["themes"]):
            expected = hex_color(Project(ROOT).compose(mode, fern).values["semantic.color.accent"])
            self.assertEqual(theme["style"]["players"][0]["cursor"], expected)

    def test_shared_valid_diff_override_rejects_unreadable_retained_syntax(self):
        with tempfile.TemporaryDirectory() as tmp:
            override = Path(tmp) / "diff-search-fill.tokens.json"
            override.write_text(json.dumps({
                "$extensions": {"org.xerrion.fjord": metadata()},
                "semantic": {"color": {"diff-add-bg": {
                    "$type": "color", "$value": "{semantic.color.search-match}",
                    "$description": "Use the source-valid search fill for added diff lines.",
                }}},
            }), encoding="utf-8")
            for mode in ("night", "day", "aurora"):
                composition = Project(ROOT).compose(mode, overrides=[override])
                check_contrast(composition)
                self.assertEqual(composition.values["semantic.color.diff-add-bg"],
                                 composition.values["semantic.color.search-match"])
            with self.assertRaisesRegex(TokenError, r"created\.background:.*below 4\.5:1"):
                zed.bundle(ROOT, overrides=[override])

    def test_bundle_and_reports_are_deterministic(self):
        first = zed.bundle(ROOT)
        self.assertEqual(first, zed.bundle(ROOT))
        self.assertEqual(json.loads(first["report.json"]), self.report)
        self.assertEqual(set(first), {"themes/fjord.json", "report.json", "extension.toml", "LICENSE"})

    def test_cli_preserves_sources_and_does_not_edit_settings(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "generated"
            command = [sys.executable, str(ROOT / "adapters/zed/generate.py"), "--output-dir", str(output)]
            result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((output / "themes/fjord.json").read_bytes(), zed.bundle(ROOT)["themes/fjord.json"])
            source = ROOT / "adapters/zed/adapter.json"
            before = source.read_bytes()
            result = subprocess.run(command[:-1] + [str(ROOT / "adapters/zed")], cwd=ROOT,
                                    text=True, capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("must not replace", result.stderr)
            self.assertEqual(source.read_bytes(), before)
            self.assertFalse((output / "settings.json").exists())

    def test_cli_accepts_relative_root_from_another_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            (directory / "project").symlink_to(ROOT, target_is_directory=True)
            command = [sys.executable, str(ROOT / "adapters/zed/generate.py"),
                       "--root", "project", "--output-dir", "generated"]
            result = subprocess.run(command, cwd=directory, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((directory / "generated/themes/fjord.json").read_bytes(),
                             zed.bundle(ROOT)["themes/fjord.json"])


if __name__ == "__main__":
    unittest.main()
