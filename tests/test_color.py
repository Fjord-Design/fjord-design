import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from fjord_tokens.color import in_srgb_gamut, map_oklch_to_srgb, oklch_to_srgb, parse_oklch, srgb_to_oklab, srgb_to_oklch
from fjord_tokens.dtcg import EXTENSION, Resolver, TokenError, color_from_hex, flatten, hex_color
from fjord_tokens.palette import GROUPS, build_palette, render_palette
from fjord_tokens.project import Project, check_contrast, contrast_value, validate_project


ROOT = Path(__file__).resolve().parents[1]
SEEDS = ["oklch(32.9478% 0.034021 247.264)", "oklch(20.8843% 0.018341 245.245)",
         "oklch(34.9975% 0.034292 250.45)", "oklch(76.0906% 0.065221 71.267)",
         "oklch(86.9933% 0.028271 46.055)", "oklch(75.9325% 0.065969 10.27)",
         "oklch(86.9339% 0.02788 129.841)"]


class ColorTests(unittest.TestCase):
    def test_published_red_oklab_vector(self):
        for actual, expected in zip(srgb_to_oklab(color_from_hex("#ff0000")), (0.6279553606, 0.2248630611, 0.1258462985)):
            self.assertAlmostEqual(actual, expected, places=9)

    def test_srgb_roundtrip_preserves_approved_palette(self):
        for mode in ("night", "day", "aurora"):
            composition = Project(ROOT).compose(mode)
            for name, value in composition.values.items():
                if composition.tokens[name]["$type"] != "color":
                    continue
                with self.subTest(mode=mode, token=name):
                    converted = map_oklch_to_srgb(srgb_to_oklch(value))
                    self.assertEqual(hex_color(converted.color)[:7], hex_color(value)[:7])

    def test_out_of_gamut_reduces_chroma_and_keeps_lightness_hue(self):
        value = (0.7, 0.4, 30)
        self.assertFalse(in_srgb_gamut(oklch_to_srgb(value)))
        result = map_oklch_to_srgb(value)
        self.assertTrue(result.mapped)
        self.assertLess(result.chroma, value[1])
        self.assertTrue(all(0 <= v <= 1 for v in result.color["components"]))
        lightness, _, hue = srgb_to_oklch(result.color)
        self.assertAlmostEqual(lightness, 0.7, places=6)
        self.assertAlmostEqual(hue, 30, places=4)
        self.assertEqual(result, map_oklch_to_srgb(value))

    def test_endpoints_and_hue_wrap(self):
        self.assertEqual(hex_color(map_oklch_to_srgb((0, 0.4, 120)).color), "#000000")
        self.assertEqual(hex_color(map_oklch_to_srgb((1, 0.4, 120)).color), "#ffffff")
        self.assertEqual(map_oklch_to_srgb((0.7, 0.1, -30)), map_oklch_to_srgb((0.7, 0.1, 330)))

    def test_invalid_or_nonfinite_coordinates_are_rejected(self):
        for value in ((-0.1, 0.1, 30), (1.1, 0.1, 30), (0.5, -0.1, 30), (0.5, 1.1, 30),
                      (True, 0.1, 30), (0.5, float("nan"), 30), (0.5, 0.1, float("inf"))):
            with self.subTest(value=value), self.assertRaises(TokenError):
                map_oklch_to_srgb(value)
        for text in ("oklch(50% .1 20)-", "oklch(101% 0.1 20)", "oklch(50% NaN 20)", "oklch(50% 0.1 none)"):
            with self.subTest(text=text), self.assertRaises(TokenError):
                parse_oklch(text)

    def test_hex_rounds_half_up_and_rejects_inconsistent_hint(self):
        color = {"colorSpace": "srgb", "components": [0.5, 0.5 / 255, 1], "alpha": 1}
        self.assertEqual(hex_color(color), "#8001ff")
        color["hex"] = "#ffffff"
        with self.assertRaisesRegex(TokenError, "must match the rounded"):
            Resolver(flatten({"primitive": {"color": {"test": {"$type": "color", "$value": color,
                                                               "$description": "Rounding test."}}}})).all()
        color["hex"] = "#8001FF"
        Resolver(flatten({"primitive": {"color": {"test": {"$type": "color", "$value": color,
                                                           "$description": "Matching fallback."}}}})).all()

    def test_hex_rounding_cannot_hide_a_contrast_failure(self):
        composition = Project(ROOT).compose("day")
        foreground = {"colorSpace": "srgb", "components": [0.465] * 3, "alpha": 1}
        background = color_from_hex("#ffffff")
        self.assertGreater(contrast_value(foreground, background), 4.5)
        self.assertLess(contrast_value(color_from_hex(hex_color(foreground)), background), 4.5)
        composition.values["semantic.color.ink"] = foreground
        composition.values["semantic.color.bg-sunken"] = background
        with self.assertRaisesRegex(TokenError, "HEX fallback.*below 4.5"):
            check_contrast(composition)


class PaletteTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory(prefix="fjord-palette-")
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        shutil.copytree(ROOT / "tokens", self.root / "tokens")
        (self.root / "docs").mkdir()
        shutil.copy2(ROOT / "docs/platforms.md", self.root / "docs/platforms.md")

    def test_original_seeds_and_every_rendered_pair_are_retained(self):
        report = build_palette(ROOT)
        self.assertEqual([seed["reference"] for seed in report["seeds"]], SEEDS)
        self.assertEqual(set(report["modes"]), {"night", "day", "aurora"})
        self.assertEqual(sum(len(mode["contrastPairs"]) for mode in report["modes"].values()), 430)
        for mode in report["modes"].values():
            self.assertTrue(set(name for names in GROUPS.values() for name in names) <= mode["colors"].keys())
            for pair in mode["contrastPairs"]:
                self.assertGreaterEqual(pair["sourceRatio"], pair["minimum"])
                self.assertGreaterEqual(pair["hexRatio"], pair["minimum"])

    def test_seed_metadata_is_validated_before_build(self):
        path = self.root / "tokens/base.tokens.json"
        document = json.loads(path.read_text())
        document["$extensions"][EXTENSION]["designSeeds"]["primary"][0] += "-"
        path.write_text(json.dumps(document))
        with self.assertRaisesRegex(TokenError, "invalid Fjord OKLCH seed"):
            validate_project(self.root)

    def test_palette_is_deterministic_and_offline(self):
        first, second = build_palette(ROOT), build_palette(ROOT)
        self.assertEqual(first, second)
        html = render_palette(first)
        self.assertEqual(html, render_palette(second))
        self.assertIn('data-mode="night"', html)
        self.assertIn('data-mode="day"', html)
        self.assertIn('data-mode="aurora"', html)
        self.assertNotIn("{{", html)
        self.assertNotIn("https://", html)
        self.assertNotIn("http://", html)

    def test_palette_follows_the_contract_base_path(self):
        (self.root / "tokens/base.tokens.json").rename(self.root / "tokens/source.tokens.json")
        path = self.root / "tokens/contract.json"
        contract = json.loads(path.read_text())
        contract["base"] = "source.tokens.json"
        path.write_text(json.dumps(contract))
        self.assertEqual([seed["reference"] for seed in build_palette(self.root)["seeds"]], SEEDS)

    def test_palette_cli_cannot_overwrite_sources(self):
        target = self.root / "docs/platforms.md"
        original = target.read_bytes()
        result = subprocess.run([sys.executable, "-m", "fjord_tokens", "palette", "--root", str(self.root),
                                 "--output", str(target)], text=True, capture_output=True)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(target.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
