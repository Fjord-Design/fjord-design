"""Export one validated composition to portable data formats."""

from fjord.dtcg import TokenError, hex_color, nest
from fjord.fonts import family_css
from fjord.project import check_contrast, metadata


def resolved_document(composition):
    return {
        "metadata": dict(metadata(), mode=composition.mode, colorSpace="srgb"),
        "tokens": {
            path: {
                "type": composition.tokens[path]["$type"],
                "value": composition.values[path],
                "source": composition.provenance[path],
                "original": composition.tokens[path]["$value"],
            }
            for path in sorted(composition.tokens)
        },
    }


def format_number(value):
    return str(value)


def dimension(value):
    return format_number(value["value"]) + value["unit"]


def css_value(kind, value):
    if kind == "color":
        return hex_color(value)
    if kind in ("dimension", "duration"):
        return dimension(value)
    if kind == "fontFamily":
        return family_css(value)
    if kind in ("fontWeight", "number"):
        return format_number(value)
    if kind == "cubicBezier":
        return "cubic-bezier(" + ", ".join(map(format_number, value)) + ")"
    if kind == "shadow":
        layers = [value] if isinstance(value, dict) else value
        return ", ".join(
            ("inset " if item.get("inset") else "")
            + " ".join(
                dimension(item[key]) for key in ("offsetX", "offsetY", "blur", "spread")
            )
            + " "
            + hex_color(item["color"])
            for item in layers
        )
    raise TokenError(f"unsupported CSS export type {kind}")


def css_document(composition):
    properties = {}
    for path in sorted(composition.values):
        kind, value = composition.tokens[path]["$type"], composition.values[path]
        name = "--fjord-token-" + path.replace(".", "-").replace("$", r"\$")
        values = (
            {name: css_value(kind, value)}
            if kind != "typography"
            else {
                name + "-font-family": css_value("fontFamily", value["fontFamily"]),
                name + "-font-size": css_value("dimension", value["fontSize"]),
                name + "-font-weight": css_value("fontWeight", value["fontWeight"]),
                name + "-letter-spacing": css_value(
                    "dimension", value["letterSpacing"]
                ),
                name + "-line-height": css_value("number", value["lineHeight"]),
            }
        )
        for key, item in values.items():
            if key in properties:
                raise TokenError(f"CSS token name collision: {key}")
            properties[key] = item
    header = f"/* Fjord contract {metadata()['contractVersion']}; mode {composition.mode}; sRGB HEX */\n:root {{\n"
    return (
        header
        + "".join(f"  {key}: {value};\n" for key, value in properties.items())
        + "}\n"
    )


def export(composition, format):
    check_contrast(composition)
    if format == "dtcg":
        return nest(
            composition.tokens,
            dict(metadata(), mode=composition.mode, colorSpace="srgb"),
        )
    if format == "resolved":
        return resolved_document(composition)
    if format == "css":
        return css_document(composition)
    raise TokenError(f"unsupported export format {format}")
