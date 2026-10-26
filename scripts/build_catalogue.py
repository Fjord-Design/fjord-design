"""Generate catalogue inspection data from validated Fjord compositions."""

from pathlib import Path

from fjord_tokens.cli import ROOT, write_output
from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import hex_color
from fjord_tokens.export import resolved_document
from fjord_tokens.project import Project, check_contrast, validate_project


def build(root=ROOT):
    validate_project(root)
    project = Project(root)
    result = {}
    for identity, theme in (("fjord", None), ("fern", Path(root) / "examples/themes/fern.tokens.json")):
        result[identity] = {}
        for mode in THEMES:
            composition = project.compose(mode, theme)
            check_contrast(composition)
            document = resolved_document(composition)
            for path, item in document["tokens"].items():
                if item["type"] == "color":
                    item["hex"] = hex_color(composition.values[path])
            result[identity][mode] = document
    return result


if __name__ == "__main__":
    destination = ROOT / "dist/catalogue-tokens.json"
    forbidden = [ROOT / name for name in ("tokens", "src", "docs", "assets", "examples", "tests", "reference", "scripts", ".git")]
    write_output(destination, build(), forbidden)
    print(f"Built catalogue token data: {destination}")
