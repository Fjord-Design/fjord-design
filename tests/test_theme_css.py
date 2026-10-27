import json
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest

from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import TokenError, hex_color
from fjord_tokens.project import Project, metadata
from fjord_tokens.theme_css import theme_css

ROOT = Path(__file__).resolve().parents[1]
FERN = ROOT / "examples/themes/fern.tokens.json"


class ThemeCssTests(unittest.TestCase):
    def test_scoped_colors_match_composed_identity_in_each_mode(self):
        css = theme_css(ROOT, "new-fern", FERN)
        self.assertEqual(css, theme_css(ROOT, "new-fern", FERN))
        for mode in THEMES:
            block = next(line for line in css.splitlines() if line.startswith(f'[data-fjord-mode="{mode}"]'))
            composed = Project(ROOT).compose(mode, FERN)
            self.assertIn('[data-fjord-identity="new-fern"]', block)
            self.assertIn('--fjord-accent:' + hex_color(composed.values["semantic.color.accent"]) + ';', block)
            self.assertIn('--fjord-button-primary-background:' + hex_color(composed.values["component.button.primary.background"]) + ';', block)
            self.assertIn('--fjord-syn-comment:' + hex_color(composed.values["semantic.color.syn-comment"]) + ';', block)
        self.assertIn('@media (prefers-color-scheme:light)', css)
        self.assertIn('[data-fjord-preference="system"]', css)
        self.assertNotIn(':root', css)

    def test_documented_cli_outputs(self):
        with tempfile.TemporaryDirectory() as tmp:
            css = Path(tmp) / "custom.theme.css"
            resolved = Path(tmp) / "custom.resolved.json"
            for args in (["theme-css", "--identity", "custom", "--theme", str(FERN), "--output", str(css)],
                         ["export", "--mode", "day", "--format", "resolved", "--theme", str(FERN), "--output", str(resolved)]):
                result = subprocess.run([sys.executable, "-m", "fjord_tokens", *args], cwd=ROOT, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
            expected = Project(ROOT).compose("day", FERN)
            document = json.loads(resolved.read_text())
            self.assertEqual(document["tokens"]["semantic.color.accent"]["value"], expected.values["semantic.color.accent"])
            self.assertIn(hex_color(expected.values["semantic.color.accent"]), css.read_text())

    def test_invalid_identity_and_non_color_override_are_rejected(self):
        for name in ("", "../bad", 'a"] { color: red; }', "CamelCase"):
            with self.subTest(name=name), self.assertRaises(TokenError):
                theme_css(ROOT, name, FERN)
        with tempfile.TemporaryDirectory() as tmp:
            override = Path(tmp) / "size.json"
            override.write_text(json.dumps({"$extensions": {"org.xerrion.fjord": metadata()}, "semantic": {"spacing": {
                "space-2": {"$type": "dimension", "$value": {"value": 100, "unit": "px"}, "$description": "Changed spacing"}}}}))
            with self.assertRaisesRegex(TokenError, "semantic colour overrides only"):
                theme_css(ROOT, "sample", overrides=[override])


if __name__ == "__main__":
    unittest.main()
