"""Build colour scales and contrast evidence from the approved token source."""

from html import escape
from pathlib import Path

from .color import GAMUT_STEPS, GAMUT_TOLERANCE, design_seed_values, map_oklch_to_srgb, parse_oklch, srgb_to_oklch
from .contract import THEMES
from .dtcg import EXTENSION, TokenError, color_from_hex, hex_color, read_json
from .project import Project, check_contrast, contrast_pairs, contrast_value, metadata, validate_project


GROUPS = {
    "Surfaces": "bg-sunken bg-base bg-surface bg-raised bg-hover bg-active".split(),
    "Text and boundaries": "ink ink-muted ink-subtle ink-disabled line line-strong focus-ring".split(),
    "Interaction": "accent accent-hover on-accent accent-soft accent-ink selection".split(),
    "Accents": "frost aurora lilac sage fern latte foam rose".split(),
    "Status": "info info-soft success success-soft warning warning-soft danger danger-soft on-danger".split(),
}
SEED_TARGETS = "bg-hover bg-base bg-active latte foam rose sage".split()


def build_palette(root, theme=None, overrides=()):
    validate_project(root)
    project = Project(root)
    source = read_json(project.source(project.contract.get("base")))["$extensions"][EXTENSION].get("designSeeds")
    seeds = []
    for original, role in zip(design_seed_values(source), SEED_TARGETS):
        coordinates = parse_oklch(original)
        converted = map_oklch_to_srgb(coordinates)
        seeds.append({"reference": original, "relatedRole": f"semantic.color.{role}", "oklch": coordinates,
                      "srgb": converted.color, "hex": hex_color(converted.color),
                      "gamutMapped": converted.mapped, "mappedChroma": converted.chroma})
    modes = {}
    for mode in THEMES:
        composition = project.compose(mode, theme, overrides)
        check_contrast(composition)
        colors = {}
        for path, value in sorted(composition.values.items()):
            if not path.startswith("semantic.color."):
                continue
            name = path.removeprefix("semantic.color.")
            alias = composition.tokens[path]["$value"]
            colors[name] = {"srgb": value, "hex": hex_color(value), "oklch": srgb_to_oklch(value),
                            "source": alias if isinstance(alias, str) else "literal"}
        pairs = []
        for foreground, background, minimum in contrast_pairs(composition):
            first, second = composition.values[foreground], composition.values[background]
            pairs.append({"foreground": foreground, "background": background, "minimum": minimum,
                          "sourceRatio": contrast_value(first, second),
                          "hexRatio": contrast_value(color_from_hex(hex_color(first)), color_from_hex(hex_color(second)))})
        modes[mode] = {"colors": colors, "contrastPairs": pairs}
    return {"metadata": metadata(), "sourceColorSpace": "srgb", "inspectionColorSpace": "oklch",
            "fallback": {"gamutMapping": "constant lightness and hue; binary-search chroma reduction",
                         "iterations": GAMUT_STEPS, "gamutTolerance": GAMUT_TOLERANCE,
                         "rounding": "floor(component * 255 + 0.5)", "contrast": "unrounded ratios on final HEX components"},
            "seeds": seeds, "groups": GROUPS, "modes": modes}


def format_oklch(value):
    return f"oklch({value[0] * 100:.4f}% {value[1]:.6f} {value[2]:.3f})"


def swatch(color):
    value = color_from_hex(color)
    labels = [color_from_hex(c) for c in ("#000000", "#ffffff")]
    label = max(labels, key=lambda c: contrast_value(c, value))
    ratio = contrast_value(label, value)
    return (f'<div class="swatch" style="background:{color};color:{hex_color(label)}">'
            f'<span aria-hidden="true">Aa</span><span class="sample-ratio">Label {ratio:.2f}:1</span></div>')


def render_palette(report):
    template = Path(__file__).with_name("templates") / "palette.html"
    css = []
    panels = []
    for mode, data in report["modes"].items():
        colors = data["colors"]
        css_names = [name for names in GROUPS.values() for name in names]
        css.append(f'html[data-theme="{mode}"] {{' + "".join(f'--{name}: {colors[name]["hex"]};' for name in css_names) + "}")
        sections = []
        for title, names in report["groups"].items():
            cards = []
            for name in names:
                color = colors[name]
                cards.append('<article class="color-card">' + swatch(color["hex"])
                             + f'<h3>{escape(name)}</h3><code class="hex">{color["hex"]}</code>'
                             + f'<code class="coordinate">{format_oklch(color["oklch"])}</code>'
                             + f'<small>{escape(color["source"])}</small></article>')
            sections.append(f'<section class="scale"><h2>{title}</h2><div class="swatches">{"".join(cards)}</div></section>')
        rows = []
        for pair in data["contrastPairs"]:
            rows.append('<tr>' + f'<td><code>{escape(pair["foreground"])}</code></td>'
                        + f'<td><code>{escape(pair["background"])}</code></td><td>{pair["minimum"]:g}:1</td>'
                        + f'<td>{pair["sourceRatio"]:.4f}:1</td><td>{pair["hexRatio"]:.4f}:1</td><td>Pass</td></tr>')
        count = len(rows)
        panels.append(f'<div class="mode-panel" data-mode="{mode}" {"hidden" if mode != "night" else ""}>'
                      + f'<p class="mode-summary">{mode.title()}: {len(colors)} colour roles. {count} contrast pairs pass before and after HEX rounding.</p>'
                      + "".join(sections) + f'<details><summary>Inspect all {count} contrast pairs</summary>'
                      + '<div class="table-scroll" role="region" aria-label="Contrast evidence" tabindex="0"><table>'
                      + '<thead><tr><th>Foreground</th><th>Background</th><th>Minimum</th><th>sRGB</th><th>HEX</th><th>Result</th></tr></thead>'
                      + f'<tbody>{"".join(rows)}</tbody></table></div></details></div>')
    references = []
    for seed in report["seeds"]:
        references.append('<article class="seed-card">' + swatch(seed["hex"])
                          + f'<code>{escape(seed["reference"])}</code><code>{seed["hex"]}</code>'
                          + f'<small>Related role: {escape(seed["relatedRole"])}</small>'
                          + f'<small>Gamut reduction: {"yes" if seed["gamutMapped"] else "not needed"}</small></article>')
    html = template.read_text(encoding="utf-8")
    for name, content in {"CSS": "\n".join(css), "PANELS": "\n".join(panels), "SEEDS": "\n".join(references)}.items():
        html = html.replace("{{" + name + "}}", content)
    return html
