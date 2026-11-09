from pathlib import Path
import unittest

from fjord.code_roles import ROLES, check_syntax_distances, oklab, syntax_distances, validate_bindings
from fjord.contract import ANSI, MODES
from fjord.dtcg import TokenError, color_from_hex
from fjord.project import Project, check_contrast


ROOT = Path(__file__).resolve().parents[1]


class CodeRoleTests(unittest.TestCase):
    def test_every_binding_resolves_in_every_mode(self):
        project = Project(ROOT)
        for mode in MODES:
            with self.subTest(mode=mode):
                composed = project.compose(mode)
                validate_bindings(composed)
                self.assertGreater(check_contrast(composed), 100)

    def test_binding_without_a_colour_source_fails(self):
        composed = Project(ROOT).compose("night")
        del composed.tokens["semantic.color.git-added"]
        with self.assertRaisesRegex(TokenError, "expected colour token semantic.color.git-added"):
            validate_bindings(composed)

    def test_status_and_diff_roles_have_distinct_text_markers(self):
        for prefix, count in (("git-", 7), ("diagnostic-", 4)):
            markers = [role.marker for name, role in ROLES.items() if name.startswith(prefix)]
            self.assertEqual(len(markers), count)
            self.assertEqual(len(set(markers)), count)
            self.assertTrue(all(marker.strip() for marker in markers))
        self.assertEqual([ROLES[f"diff-{state}"].marker[0] for state in ("added", "changed", "deleted")], ["+", "~", "-"])

    def test_all_ansi_slots_are_bound_in_order(self):
        self.assertEqual([ROLES[name].token for name in ANSI], [f"semantic.color.{name}" for name in ANSI])
        self.assertEqual(ANSI[0], "ansi-black")
        self.assertEqual(ANSI[8], "ansi-bright-black")
        self.assertEqual(ANSI[15], "ansi-bright-white")

    def test_oklab_reference_red(self):
        expected = (0.6279553606, 0.2248630611, 0.1258462985)
        for actual, reference in zip(oklab(color_from_hex("#ff0000")), expected):
            self.assertAlmostEqual(actual, reference, places=8)

    def test_syntax_distance_thresholds_pass_every_mode(self):
        for mode in MODES:
            composed = Project(ROOT).compose(mode)
            self.assertEqual(len(list(syntax_distances(composed))), 36)
            check_syntax_distances(composed)

    def test_readable_but_indistinguishable_syntax_fails(self):
        composed = Project(ROOT).compose("night")
        composed.values["semantic.color.syn-type"] = composed.values["semantic.color.syn-keyword"]
        with self.assertRaisesRegex(TokenError, "OKLab distance .* is below 7"):
            check_contrast(composed)


if __name__ == "__main__":
    unittest.main()
