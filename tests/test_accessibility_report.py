import importlib.util
from pathlib import Path
import unittest

from fjord_tokens.contract import THEMES
from fjord_tokens.project import Project, contrast_pairs

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("accessibility_report", ROOT / "scripts/accessibility_report.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class AccessibilityReportTests(unittest.TestCase):
    def test_report_matches_validated_pairs_without_rounding(self):
        report = module.build(ROOT)
        self.assertEqual(report, module.build(ROOT))
        for identity, theme in (("fjord", None), ("fern", ROOT / "examples/themes/fern.tokens.json")):
            for mode in THEMES:
                with self.subTest(identity=identity, mode=mode):
                    composition = Project(ROOT).compose(mode, theme)
                    expected = {(f, b, threshold) for f, b, threshold in contrast_pairs(composition)}
                    actual = report["identities"][identity][mode]
                    self.assertEqual(actual["count"], len(expected))
                    self.assertEqual({(c["foreground"], c["background"], c["minimum"]) for c in actual["checks"]}, expected)
                    self.assertTrue(all(c["srgb"] >= c["minimum"] and c["hex"] >= c["minimum"] for c in actual["checks"]))


if __name__ == "__main__":
    unittest.main()
