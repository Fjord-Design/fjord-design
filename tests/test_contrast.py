from pathlib import Path
import unittest

from fjord_tokens.dtcg import TokenError, color_from_hex
from fjord_tokens.project import Project, check_contrast, contrast_value


ROOT = Path(__file__).resolve().parents[1]


class ContrastTests(unittest.TestCase):
    def test_known_srgb_ratios(self):
        for foreground, background, expected in (
            ("#000000", "#ffffff", 21), ("#112233", "#112233", 1),
            ("#767676", "#ffffff", 4.5422249596), ("#777777", "#ffffff", 4.4780894536),
        ):
            with self.subTest(foreground=foreground):
                self.assertAlmostEqual(contrast_value(color_from_hex(foreground), color_from_hex(background)), expected)

    def test_dim_terminal_text_below_threshold_fails(self):
        for mode, color, ratio in (("night", "#738290", "4.498225"), ("day", "#687683", "4.096792")):
            with self.subTest(mode=mode):
                composed = Project(ROOT).compose(mode)
                composed.values["semantic.color.ansi-bright-black"] = color_from_hex(color)
                with self.assertRaisesRegex(TokenError, ratio + ":1 is below 4.5:1"):
                    check_contrast(composed)

    def test_focus_and_status_contrast_are_checked(self):
        for role, background, minimum in (("focus-ring", "bg-base", "3"), ("danger", "danger-soft", "4.5")):
            with self.subTest(role=role):
                composed = Project(ROOT).compose("day")
                composed.values[f"semantic.color.{role}"] = composed.values[f"semantic.color.{background}"]
                with self.assertRaisesRegex(TokenError, f"{role}.*below {minimum}:1"):
                    check_contrast(composed)

    def test_comments_require_the_stricter_night_ratio(self):
        composed = Project(ROOT).compose("night")
        composed.values["semantic.color.syn-comment"] = color_from_hex("#849a99")
        with self.assertRaisesRegex(TokenError, "syn-comment.*below 7.5:1"):
            check_contrast(composed)

    def test_explicit_full_opacity_is_supported(self):
        self.assertEqual(contrast_value(color_from_hex("#000000ff"), color_from_hex("#ffffffff")), 21)


if __name__ == "__main__":
    unittest.main()
