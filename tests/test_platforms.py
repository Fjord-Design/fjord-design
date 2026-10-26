from pathlib import Path
import unittest

from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import hex_color
from fjord_tokens.project import Project
from fjord_tokens.quality import Report, check_platforms


ROOT = Path(__file__).resolve().parents[1]


class PlatformTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.markdown = (ROOT / "docs/platforms.md").read_text(encoding="utf-8")
        cls.colors = {}
        project = Project(ROOT)
        for mode in THEMES:
            for path, value in project.compose(mode).values.items():
                if path.startswith("semantic.color."):
                    cls.colors[(path.removeprefix("semantic.color."), mode)] = hex_color(value)

    def report(self, markdown):
        report = Report()
        check_platforms(markdown, self.colors, report)
        return report

    def assert_error(self, markdown, message):
        self.assertIn(message, "\n".join(self.report(markdown).errors))

    def test_all_documented_values_match_resolved_tokens(self):
        report = self.report(self.markdown)
        self.assertEqual(report.errors, [])
        self.assertEqual(report.platform_values, 102)

    def test_invalid_or_changed_color_is_rejected(self):
        for value in ("#000000", "broken"):
            with self.subTest(value=value):
                self.assert_error(self.markdown.replace("`#788795`", f"`{value}`"),
                                  f"ansi-bright-black[night] is '{value}'; use #788795 from the resolved DTCG sources")

    def test_missing_or_duplicate_row_is_rejected(self):
        row = next(line for line in self.markdown.splitlines() if "| 8 |" in line)
        for replacement, error in (("", "missing row for ansi-bright-black"),
                                   (row + "\n" + row, "duplicate row")):
            with self.subTest(error=error):
                self.assert_error(self.markdown.replace(row, replacement), error)

    def test_unknown_token_is_rejected(self):
        self.assert_error(self.markdown.replace("`ansi-bright-black`", "`ansi-brigth-black`"),
                          "unknown colour token 'ansi-brigth-black'")

    def test_wrong_ansi_slot_is_rejected(self):
        self.assert_error(self.markdown.replace("| 8 |", "| 9 |"), "incorrect ANSI slot for ansi-bright-black")

    def test_missing_mode_column_is_rejected(self):
        self.assert_error(self.markdown.replace("| Token | Night | Day | Aurora |", "| Token | Night | Aurora |"),
                          "headers must be")

    def test_missing_tables_are_rejected(self):
        report = self.report("# Platforms\nNo colour tables.\n")
        self.assertEqual(len(report.errors), 2)
        for kind in ("core", "ansi"):
            self.assertIn(f"missing {kind} colour table", "\n".join(report.errors))

    def test_duplicate_table_is_rejected(self):
        self.assert_error(self.markdown + "\n" + self.markdown, "duplicate colour table")

    def test_missing_table_separator_is_rejected(self):
        self.assert_error(self.markdown.replace("| --- | --- | --- | --- |\n", "", 1),
                          "missing or malformed Markdown table separator")

    def test_missing_cell_is_rejected(self):
        self.assert_error(self.markdown.replace("| `#788795` | `#5f6d7a` |", "| `#788795` |"),
                          "expected 5 columns, got 4")

    def test_reordered_mode_columns_use_headers(self):
        lines = []
        for line in self.markdown.splitlines():
            cells = line.split("|")
            if len(cells) == 6:
                cells[2], cells[3] = cells[3], cells[2]
                line = "|".join(cells)
            lines.append(line)
        report = self.report("\n".join(lines))
        self.assertEqual(report.errors, [])
        self.assertEqual(report.platform_values, 102)


if __name__ == "__main__":
    unittest.main()
