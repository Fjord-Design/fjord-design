"""Export one validated composition to portable formats and WezTerm colours."""

from .contract import ANSI
from .dtcg import TokenError, hex_color, nest
from .fonts import family_css
from .project import check_contrast, metadata


def resolved_document(composition):
    return {
        "metadata": dict(metadata(), mode=composition.mode, colorSpace="srgb"),
        "tokens": {
            path: {"type": composition.tokens[path]["$type"], "value": composition.values[path],
                   "source": composition.provenance[path], "original": composition.tokens[path]["$value"]}
            for path in sorted(composition.tokens)
        },
    }


def number(value):
    return str(value)


def dimension(value):
    return number(value["value"]) + value["unit"]


def css_value(kind, value):
    if kind == "color":
        return hex_color(value)
    if kind in ("dimension", "duration"):
        return dimension(value)
    if kind == "fontFamily":
        return family_css(value)
    if kind in ("fontWeight", "number"):
        return number(value)
    if kind == "cubicBezier":
        return "cubic-bezier(" + ", ".join(map(number, value)) + ")"
    if kind == "shadow":
        return ", ".join(("inset " if item.get("inset") else "")
                         + " ".join(dimension(item[key]) for key in ("offsetX", "offsetY", "blur", "spread"))
                         + " " + hex_color(item["color"]) for item in value)
    raise TokenError(f"unsupported CSS export type {kind}")


def css_document(composition):
    properties = {}
    for path in sorted(composition.values):
        kind, value = composition.tokens[path]["$type"], composition.values[path]
        name = "--fjord-token-" + path.replace(".", "-")
        values = {name: css_value(kind, value)} if kind != "typography" else {
            name + "-font-family": css_value("fontFamily", value["fontFamily"]),
            name + "-font-size": css_value("dimension", value["fontSize"]),
            name + "-font-weight": css_value("fontWeight", value["fontWeight"]),
            name + "-letter-spacing": css_value("dimension", value["letterSpacing"]),
            name + "-line-height": css_value("number", value["lineHeight"]),
        }
        for key, item in values.items():
            if key in properties:
                raise TokenError(f"CSS token name collision: {key}")
            properties[key] = item
    header = f"/* Fjord contract {metadata()['contractVersion']}; mode {composition.mode}; sRGB HEX */\n:root {{\n"
    return header + "".join(f"  {key}: {value};\n" for key, value in properties.items()) + "}\n"


def wezterm_document(composition):
    values = composition.values
    def color(name):
        return hex_color(values["semantic.color." + name])
    fields = {
        "foreground": "ink", "background": "bg-base", "cursor_bg": "accent", "cursor_border": "accent",
        "cursor_fg": "on-accent", "selection_bg": "selection", "selection_fg": "ink",
    }
    header = f"# Fjord contract {metadata()['contractVersion']}; mode {composition.mode}; sRGB HEX\n[colors]\n"
    lines = [f'{field} = "{color(token)}"\n' for field, token in fields.items()]
    for field, names in (("ansi", ANSI[:8]), ("brights", ANSI[8:])):
        lines.append(field + " = [" + ", ".join('"' + color(name) + '"' for name in names) + "]\n")
    return header + "".join(lines) + f'\n[metadata]\nname = "Fjord {composition.mode.title()}"\nauthor = "Fjord contributors"\n'


def export(composition, format):
    check_contrast(composition)
    if format == "dtcg":
        return nest(composition.tokens, dict(metadata(), mode=composition.mode, colorSpace="srgb"))
    if format == "resolved":
        return resolved_document(composition)
    if format == "css":
        return css_document(composition)
    if format == "wezterm":
        return wezterm_document(composition)
    raise TokenError(f"unsupported export format {format}")
