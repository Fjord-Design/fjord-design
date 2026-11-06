"""Build Zed theme files from the shared declarative adapter mapping."""

import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

from fjord_tokens.adapters import map_adapter, write_adapter_files
from fjord_tokens.dtcg import TokenError, color_from_hex, hex_color, read_json
from fjord_tokens.project import Project, contrast_value, metadata


ROOT = Path(__file__).resolve().parents[2]
HEX = re.compile(r"#[0-9a-f]{6}")
HIGHLIGHTS = ("search.match_background", "editor.document_highlight.read_background",
              "editor.document_highlight.write_background", "editor.document_highlight.bracket_background",
              "player.selection")


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n").encode("utf-8")


def schema_document(root):
    directory = Path(root) / "adapters/zed/schema"
    provenance = read_json(directory / "provenance.json")
    data = (directory / "themes-v0.2.0.json").read_bytes()
    if hashlib.sha256(data).hexdigest() != provenance["sha256"]:
        raise TokenError("Zed schema checksum differs from pinned provenance; review the schema update")
    return read_json(directory / "themes-v0.2.0.json"), provenance


def validate_family(family, schema):
    """Check emitted values against the pinned schema; reject unknown style keys too.

    This deliberately validates only JSON Schema keywords used by the emitted Zed
    family, not arbitrary third-party schemas. In contrast to Zed's permissive
    deserializer, unknown fields in fixed objects are errors so typos cannot pass.
    """
    def visit(value, rule, location):
        if "anyOf" in rule:
            for candidate in rule["anyOf"]:
                try:
                    check(value, candidate, location)
                    return
                except TokenError:
                    pass
            raise TokenError(f"{location}: value does not match the Zed schema")
        kinds = rule.get("type", [])
        kinds = [kinds] if isinstance(kinds, str) else kinds
        matches = {"object": isinstance(value, dict), "array": isinstance(value, list),
                   "string": isinstance(value, str), "null": value is None,
                   "integer": isinstance(value, int) and not isinstance(value, bool),
                   "number": isinstance(value, (int, float)) and not isinstance(value, bool),
                   "boolean": isinstance(value, bool)}
        if kinds and not any(matches[kind] for kind in kinds):
            raise TokenError(f"{location}: expected Zed schema type {kinds}")
        if "enum" in rule and value not in rule["enum"]:
            raise TokenError(f"{location}: invalid Zed schema enum {value!r}")
        if isinstance(value, dict):
            missing = set(rule.get("required", [])) - set(value)
            if missing:
                raise TokenError(f"{location}: missing Zed schema fields {sorted(missing)}")
            properties = rule.get("properties", {})
            extra = rule.get("additionalProperties")
            for key, item in value.items():
                child = properties.get(key, extra)
                if not isinstance(child, dict):
                    raise TokenError(f"{location}.{key}: unknown Zed schema field")
                check(item, child, f"{location}.{key}")
        elif isinstance(value, list):
            for index, item in enumerate(value):
                check(item, rule["items"], f"{location}[{index}]")

    def check(value, rule, location):
        while "$ref" in rule:
            reference = rule["$ref"]
            if not reference.startswith("#/"):
                raise TokenError(f"{location}: unsupported external schema reference")
            rule = schema
            for segment in reference[2:].split("/"):
                rule = rule[segment]
        visit(value, rule, location)

    check(family, schema, "theme")
    for theme in family["themes"]:
        style = theme["style"]
        colors = {key: value for key, value in style.items()
                  if key not in ("syntax", "players", "background.appearance")}
        colors.update({"syntax." + key: value["color"] for key, value in style.get("syntax", {}).items()})
        for index, player in enumerate(style.get("players", [])):
            colors.update({f"players[{index}].{key}": value for key, value in player.items()})
        for key, value in colors.items():
            if not isinstance(value, str) or not HEX.fullmatch(value):
                raise TokenError(f"{theme['name']} {key}: emitted Zed colors must be opaque 8-bit HEX")


def ratio(foreground, background):
    return contrast_value(color_from_hex(foreground), color_from_hex(background))


def retained_foregrounds(mapped, mode):
    fields = [key for key in mapped if key.startswith("syntax.")]
    fields += ["editor.foreground",
               "terminal.foreground", "terminal.dim_foreground", "terminal.bright_foreground"]
    fields += [key for key in mapped if key.startswith("terminal.ansi.")
               and key != "terminal.ansi.background"
               and not (key == "terminal.ansi.black" and mode != "day")]
    return {key: mapped[key] for key in sorted(set(fields))}


def safe_highlight(desired, foregrounds, appearance):
    """Keep the desired hue, moving toward black/white until rounded HEX passes.

    Zed keeps syntax and terminal foregrounds under highlights, while Fjord's
    portable highlights assume ink text. A finite 8-bit search finds the first
    passing fill and preserves the actual emitted contrast in the report.
    """
    original = color_from_hex(desired)["components"]
    endpoint = 1 if appearance == "light" else 0
    for step in range(1025):
        amount = step / 1024
        candidate = hex_color({"components": [channel * (1 - amount) + endpoint * amount
                                                for channel in original], "alpha": 1})
        if all(ratio(color, candidate) >= 4.5 for color in foregrounds.values()):
            return candidate, amount
    raise TokenError("Zed highlight cannot retain all foregrounds at 4.5:1; adjust the adapter mapping")


def build(root=ROOT, theme=None, overrides=()):
    root = Path(root).resolve()
    mapped_modes, report = map_adapter(root, root / "adapters/zed/adapter.json", theme, overrides)
    schema, provenance = schema_document(root)
    family = {"name": "Fjord", "author": "Fjord contributors", "themes": []}
    report = deepcopy(report)
    report["zedSchema"] = provenance
    report["tokenContract"] = metadata()["contractVersion"]
    report["contrast"] = {}
    report["substitutions"] = {}
    report["visibility"] = {}
    report["limitations"] = [
        "ANSI black is a palette slot, not ordinary text on the dark background; it is checked as text in Day.",
        "Zed has one search-match fill; independent current-search colour and selected-text foreground are unavailable.",
        "Only the local player's colors are supplied; remote collaboration colors use Zed's behavior.",
        "Theme files do not configure font families, ligatures, cursor shape or keyboard behavior.",
        "Contrast measurements cover emitted opaque HEX values. Zed can adjust editor text via APCA, fade unfocused editor selections to 50% opacity, and composite terminal SGR DIM text at reduced opacity.",
    ]
    for mode, mapped in mapped_modes.items():
        appearance = "light" if mode == "day" else "dark"
        checks, substitutions = [], []
        values = dict(mapped)
        previous_selection = None
        if mode == "aurora":
            # The shared selection also highlights ANSI terminal text. Keeping
            # the darkest hint slot unchanged forced the fill almost onto the
            # base surface, making native Aurora selections hard to identify.
            fallback_role = "semantic.color.ink-subtle"
            previous_selection, _ = safe_highlight(mapped["player.selection"],
                                                   retained_foregrounds(mapped, mode), appearance)
            fallback = hex_color(Project(root).compose(mode, theme, overrides).values[fallback_role])
            targets = [key for key in values if key.startswith("terminal.ansi.dim_")]
            targets += ["terminal.ansi.bright_black", "terminal.dim_foreground"]
            for target in sorted(targets):
                original = values[target]
                values[target] = fallback
                if original != fallback:
                    substitutions.append({"target": target, "source": original, "emitted": fallback,
                                          "sourceRole": report["modes"][mode]["mapped"][target]["role"],
                                          "fallbackSemanticRole": fallback_role,
                                          "reason": "Use Fjord's existing secondary text role so Aurora's shared editor/terminal selection can remain visible while retaining 4.5:1 emitted text contrast."})
        foregrounds = retained_foregrounds(values, mode)
        for field in HIGHLIGHTS:
            source = values[field]
            values[field], amount = safe_highlight(source, foregrounds, appearance)
            if values[field] != source:
                substitutions.append({"target": field, "source": source, "emitted": values[field],
                                      "sourceRole": report["modes"][mode]["mapped"][field]["role"],
                                      "mixToward": "white" if appearance == "light" else "black",
                                      "mixFraction": amount,
                                      "reason": "Zed retains syntax and ANSI foregrounds on the highlight fill."})
        editor_text = {key: value for key, value in foregrounds.items() if not key.startswith("terminal.")}
        editor_text.update({key: values[key] for key in ("editor.line_number", "editor.active_line_number")})
        terminal_text = {key: value for key, value in foregrounds.items() if key.startswith("terminal.")}
        diff_text = {key: value for key, value in editor_text.items()
                     if key == "editor.foreground" or key.startswith("syntax.")}
        surfaces = {"editor.background": editor_text, "editor.active_line.background": editor_text,
                    "terminal.background": terminal_text}
        surfaces.update({key: diff_text for key in ("created.background", "deleted.background", "modified.background")})
        surfaces.update({key: foregrounds for key in HIGHLIGHTS})
        for background, text_colors in surfaces.items():
            for foreground, color in text_colors.items():
                actual = ratio(color, values[background])
                check = {"foreground": foreground, "background": background, "foregroundHex": color,
                         "backgroundHex": values[background], "ratio": actual, "minimum": 4.5,
                         "pass": actual >= 4.5}
                checks.append(check)
                if not check["pass"]:
                    raise TokenError(f"Zed {mode} {foreground} on {background}: {actual:.8f}:1 below 4.5:1")
        report["contrast"][mode] = checks
        report["substitutions"][mode] = substitutions
        if previous_selection is not None:
            base = values["editor.background"]
            report["visibility"][mode] = {
                "backgroundHex": base,
                "previousSelectionHex": previous_selection,
                "emittedSelectionHex": values["player.selection"],
                "sourceSelectionHex": mapped["player.selection"],
                "previousSelectionBaseRatio": ratio(previous_selection, base),
                "emittedSelectionBaseRatio": ratio(values["player.selection"], base),
                "sourceSelectionBaseRatio": ratio(mapped["player.selection"], base),
                "purpose": "Compare selection visibility before and after the Aurora fallback; these ratios are not a WCAG pass criterion.",
            }
        for target, value in values.items():
            report["modes"][mode]["mapped"][target]["emittedValue"] = value
        style = {key: value for key, value in values.items()
                 if not key.startswith(("syntax.", "player."))}
        style["background.appearance"] = "opaque"
        style["players"] = [{key.removeprefix("player."): value for key, value in values.items()
                             if key.startswith("player.")}]
        style["syntax"] = {key.removeprefix("syntax."): {"color": value,
                           "font_style": "italic" if key in ("syntax.comment", "syntax.comment.doc", "syntax.keyword") else "normal"}
                           for key, value in values.items() if key.startswith("syntax.")}
        family["themes"].append({"name": "Fjord " + mode.title(), "appearance": appearance, "style": style})
    validate_family(family, schema)
    return family, report


def bundle(root=ROOT, theme=None, overrides=()):
    root = Path(root).resolve()
    family, report = build(root, theme, overrides)
    directory = root / "adapters/zed"
    return {"themes/fjord.json": json_bytes(family), "report.json": json_bytes(report),
            "extension.toml": (directory / "extension.toml").read_bytes(),
            "LICENSE": (directory / "LICENSE").read_bytes()}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--root", type=Path, default=ROOT, help="Fjord source project")
    parser.add_argument("--output-dir", type=Path, help="destination; default: dist/adapters/zed")
    parser.add_argument("--theme", type=Path, help="replace the theme layer in all modes")
    parser.add_argument("--override", type=Path, action="append", default=[], help="override; repeat in precedence order")
    args = parser.parse_args(argv)
    try:
        files = bundle(args.root, args.theme, args.override)
        destination = args.output_dir or args.root / "dist/adapters/zed"
        inputs = args.override + ([args.theme] if args.theme else [])
        write_adapter_files(args.root, destination, files, inputs)
        print(f"Built Fjord Night, Day and Aurora: {destination / 'themes/fjord.json'}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
