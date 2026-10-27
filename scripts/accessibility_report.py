"""Report the exact validated token contrast pairs for each identity and mode."""

from pathlib import Path

from fjord_tokens.cli import ROOT, write_output
from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import color_from_hex, hex_color
from fjord_tokens.project import Project, check_contrast, contrast_pairs, contrast_value, metadata, validate_project


def build(root=ROOT):
    validate_project(root)
    project = Project(root)
    identities = {}
    for identity, theme in (("fjord", None), ("fern", Path(root) / "examples/themes/fern.tokens.json")):
        identities[identity] = {}
        for mode in THEMES:
            composed = project.compose(mode, theme)
            check_contrast(composed)
            checks = []
            for foreground, background, minimum in contrast_pairs(composed):
                f, b = composed.values[foreground], composed.values[background]
                fg_hex, bg_hex = hex_color(f), hex_color(b)
                checks.append({"foreground": foreground, "background": background, "minimum": minimum,
                               "srgb": contrast_value(f, b),
                               "hex": contrast_value(color_from_hex(fg_hex), color_from_hex(bg_hex)),
                               "foregroundHex": fg_hex, "backgroundHex": bg_hex})
            checks.sort(key=lambda item: (item["foreground"], item["background"]))
            identities[identity][mode] = {"count": len(checks), "checks": checks}
    return {"metadata": metadata(), "identities": identities}


if __name__ == "__main__":
    destination = ROOT / "dist/accessibility-contrast.json"
    forbidden = [ROOT / name for name in ("tokens", "src", "docs", "assets", "examples", "tests", "reference", "scripts", ".git")]
    write_output(destination, build(), forbidden)
    print(f"Built contrast report: {destination}")
