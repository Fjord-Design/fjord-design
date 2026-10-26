import importlib.util
import re
from pathlib import Path
import unittest

from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import hex_color
from fjord_tokens.project import Project

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("build_catalogue", ROOT / "scripts/build_catalogue.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class CatalogueTests(unittest.TestCase):
    def test_all_mvp_components_appear_in_story_sources(self):
        inventory = (ROOT / "docs/components/inventory.md").read_text()
        names = re.findall(r"\| ([^|]+) \| MVP v0\.1 \|", inventory)
        self.assertEqual(len(names), 23)
        stories = "\n".join(path.read_text() for path in (ROOT / "reference/svelte/src/stories").glob("*.svelte"))
        for name in names:
            with self.subTest(component=name):
                self.assertRegex(stories, r"<" + re.escape(name.replace(" ", "")) + r"(?:\s|/|>)")

    def test_catalogue_data_matches_validated_compositions(self):
        report = module.build(ROOT)
        self.assertEqual(report, module.build(ROOT))
        for identity, theme in (("fjord", None), ("fern", ROOT / "examples/themes/fern.tokens.json")):
            for mode in THEMES:
                with self.subTest(identity=identity, mode=mode):
                    composition = Project(ROOT).compose(mode, theme)
                    tokens = report[identity][mode]["tokens"]
                    self.assertEqual(set(tokens), set(composition.tokens))
                    for path, entry in tokens.items():
                        self.assertEqual(entry["value"], composition.values[path])
                        self.assertEqual(entry["original"], composition.tokens[path]["$value"])
                        self.assertEqual(entry["source"], composition.provenance[path])
                        if entry["type"] == "color":
                            self.assertEqual(entry["hex"], hex_color(composition.values[path]))
        self.assertNotEqual(report["fjord"]["night"]["tokens"]["semantic.color.accent"]["hex"],
                            report["fern"]["night"]["tokens"]["semantic.color.accent"]["hex"])


if __name__ == "__main__":
    unittest.main()
