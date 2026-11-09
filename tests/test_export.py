import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from fjord.contract import MODES
from fjord.dtcg import TokenError, hex_color, nest
from fjord.export import css_document, export
from fjord.project import Project, metadata

ROOT = Path(__file__).resolve().parents[1]


class ExportTests(unittest.TestCase):
    def test_formats_use_one_composition_and_are_deterministic(self):
        project = Project(ROOT)
        for theme in (None, ROOT / "examples/themes/fern.tokens.json"):
            for mode in MODES:
                with self.subTest(theme=theme, mode=mode):
                    composed = project.compose(mode, theme)
                    for kind in ("dtcg", "resolved", "css"):
                        self.assertEqual(export(composed, kind), export(composed, kind))
                    self.assertEqual(export(composed, "dtcg"), nest(composed.tokens, dict(metadata(), mode=mode, colorSpace="srgb")))
                    resolved = export(composed, "resolved")
                    self.assertEqual(resolved["metadata"]["mode"], mode)
                    self.assertEqual(resolved["metadata"]["colorSpace"], "srgb")
                    for name, value in composed.values.items():
                        self.assertEqual(resolved["tokens"][name]["value"], value)
                        self.assertEqual(resolved["tokens"][name]["original"], composed.tokens[name]["$value"])
                    css = export(composed, "css")
                    self.assertIn("--fjord-token-semantic-color-accent: " + hex_color(composed.values["semantic.color.accent"]) + ";", css)
                    self.assertIn("--fjord-token-semantic-typography-body-font-size: 15.0px;", css)

    def test_aliases_and_missing_roles_are_rejected_before_export(self):
        project = Project(ROOT)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "bad.json"
            path.write_text(json.dumps({"$extensions": {"org.xerrion.fjord": metadata()},
                                        "semantic": {"color": {"accent": {"$type": "color", "$value": "{semantic.color.not-real}", "$description": "Bad alias"}}}}))
            with self.assertRaises(TokenError):
                project.compose("night", overrides=[path])
        composed = project.compose("night")
        del composed.tokens["semantic.color.syn-comment"]
        with self.assertRaises(TokenError):
            export(composed, "css")

    def test_name_collisions_are_rejected(self):
        composed = Project(ROOT).compose("night")
        composed.values["semantic.color.a-b"] = composed.values["semantic.color.ink"]
        composed.tokens["semantic.color.a-b"] = composed.tokens["semantic.color.ink"]
        composed.values["semantic.color.a.b"] = composed.values["semantic.color.ink"]
        composed.tokens["semantic.color.a.b"] = composed.tokens["semantic.color.ink"]
        with self.assertRaisesRegex(TokenError, "name collision"):
            css_document(composed)

    def test_cli_cannot_replace_sources_and_is_repeatable(self):
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "night.tokens.css"
            command = [sys.executable, "-m", "fjord", "export", "--format", "css", "--output", str(output)]
            for _ in range(2):
                result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                data = output.read_bytes()
                if _:
                    self.assertEqual(data, first)
                first = data
            target = ROOT / "tokens" / "base.tokens.json"
            before = target.read_bytes()
            result = subprocess.run(command[:-1] + [str(target)], cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(target.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
