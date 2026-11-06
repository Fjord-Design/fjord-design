"""Build a local VS Code theme extension from validated Fjord semantic roles."""

import argparse
from copy import deepcopy
import json
from pathlib import Path
import re
import sys

from fjord_tokens.adapters import map_adapter, write_adapter_files
from fjord_tokens.contract import THEMES
from fjord_tokens.dtcg import TokenError, color_from_hex, hex_color, read_json
from fjord_tokens.project import contrast_value


ROOT = Path(__file__).resolve().parents[2]
HEX = re.compile(r"#[0-9a-f]{6}(?:[0-9a-f]{2})?")
SCOPES = {
    "comment": ["comment", "punctuation.definition.comment"],
    "keyword": ["keyword", "storage.type", "storage.modifier"],
    "string": ["string", "constant.other.symbol"],
    "number": ["constant.numeric", "constant.language.boolean", "constant.language.null"],
    "type": ["entity.name.type", "entity.name.class", "support.type", "support.class"],
    "function": ["entity.name.function", "support.function"],
    "property": ["variable.other.property", "meta.object-literal.key", "entity.other.attribute-name", "entity.name.tag", "support.type.property-name"],
    "variable": ["variable", "variable.parameter"],
    "operator": ["keyword.operator"],
    "punctuation": ["punctuation"],
    "regex": ["string.regexp", "constant.character.escape"],
}
TRANSLUCENT = {
    "editor.inactiveSelectionBackground": 96,
    "terminal.inactiveSelectionBackground": 96,
    "editor.selectionHighlightBackground": 0,
    "editor.wordHighlightBackground": 0,
    "editor.wordHighlightStrongBackground": 0,
    "editor.wordHighlightTextBackground": 0,
    "editor.findMatchHighlightBackground": 96,
    "diffEditor.insertedTextBackground": 96,
    "diffEditor.removedTextBackground": 96,
    "diffEditor.insertedLineBackground": 48,
    "diffEditor.removedLineBackground": 48,
}


def json_text(value):
    return json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + "\n"


def composite(foreground, background):
    foreground = color_from_hex(foreground)
    background = color_from_hex(background) if isinstance(background, str) else background
    if background.get("alpha", 1) != 1:
        raise TokenError("VS Code contrast underlay must be opaque")
    alpha = foreground.get("alpha", 1)
    return {"colorSpace": "srgb", "alpha": 1,
            "components": [front * alpha + back * (1 - alpha)
                           for front, back in zip(foreground["components"], background["components"])]}


def pair(foreground, background, underlay):
    composed = composite(background, underlay)
    foreground_color = color_from_hex(foreground)
    if foreground_color.get("alpha", 1) != 1:
        raise TokenError("VS Code checked text foreground must be opaque")
    unrounded = contrast_value(foreground_color, composed)
    emitted_background = hex_color(composed)
    rounded = contrast_value(foreground_color, color_from_hex(emitted_background))
    return unrounded, rounded, emitted_background


def terminal_find_fill(source, foregrounds, base):
    # Terminal find has no independent foreground override. Keep an accent border
    # visible even when a low-contrast hint slot requires a transparent fill.
    for alpha in range(96, -1, -1):
        candidate = source + f"{alpha:02x}"
        if all(min(pair(value, candidate, base)[:2]) >= 4.5 for value in foregrounds.values()):
            return candidate
    raise TokenError("VS Code terminal find cannot retain foregrounds at 4.5:1")


def editor_selection_fill(source, foregrounds, mode):
    # Normal vs/vs-dark themes do not render the inline-selected-text class
    # required by editor.selectionForeground. Preserve the actual syntax text.
    channels = color_from_hex(source)["components"]
    endpoint = 1 if mode == "day" else 0
    for step in range(1025):
        amount = step / 1024
        candidate = hex_color({"components": [channel * (1 - amount) + endpoint * amount
                                              for channel in channels], "alpha": 1})
        if all(min(pair(value, candidate, candidate)[:2]) >= 4.5 for value in foregrounds.values()):
            return candidate, amount
    raise TokenError("VS Code editor selection cannot retain syntax at 4.5:1")


def validate_targets(mapped, contract):
    missing = set(contract["adapterRequiredTargets"]) - set(mapped)
    if missing:
        raise TokenError(f"missing required VS Code target mappings: {sorted(missing)}")
    for target, value in mapped.items():
        if not isinstance(value, str) or not HEX.fullmatch(value) or len(value) != 7:
            raise TokenError(f"{target}: expected an opaque semantic colour, received {value!r}")
        if target.startswith("colors."):
            if target.removeprefix("colors.") not in contract["colorIds"]:
                raise TokenError(f"{target}: unknown VS Code 1.145.0 colour ID")
        elif target.startswith("tm."):
            if target.removeprefix("tm.") not in SCOPES:
                raise TokenError(f"{target}: unknown TextMate role")
        elif target.startswith("semantic."):
            selector = target.removeprefix("semantic.")
            parts = selector.split(".")
            if parts[0] not in contract["semanticTokenTypes"] or len(parts) > 2 or len(parts) == 2 and parts[1] != "readonly":
                raise TokenError(f"{target}: unsupported semantic token selector")
        else:
            raise TokenError(f"{target}: expected colors.*, tm.* or semantic.* target")


def validate_package(package, theme_paths):
    if (not isinstance(package, dict) or package.get("publisher") != "fjord"
            or package.get("name") != "fjord-themes" or package.get("version") != "0.1.0"
            or package.get("license") != "MIT" or package.get("engines") != {"vscode": "^1.145.0"}):
        raise TokenError("VS Code package must declare the pinned local Fjord theme identity and engine")
    if set(package) - {"name", "displayName", "description", "version", "publisher", "license", "engines", "categories", "repository", "contributes"}:
        raise TokenError("VS Code theme package contains unsupported executable or unrelated fields")
    contributions = package.get("contributes", {})
    if not isinstance(contributions, dict) or set(contributions) != {"themes"}:
        raise TokenError("VS Code package must contribute only themes")
    themes = contributions["themes"]
    if not isinstance(themes, list) or len(themes) != 3:
        raise TokenError("VS Code package requires Night, Day and Aurora contributions")
    for mode, contribution in zip(THEMES, themes):
        expected = {"label": "Fjord " + mode.title(), "uiTheme": "vs" if mode == "day" else "vs-dark",
                    "path": "./themes/fjord-" + mode + "-color-theme.json"}
        if contribution != expected or contribution["path"][2:] not in theme_paths:
            raise TokenError(f"VS Code {mode}: invalid theme contribution or path")


def build_bundle(root=ROOT, theme=None, overrides=()):
    root = Path(root).resolve()
    directory = root / "adapters/vscode"
    contract = read_json(directory / "contract/vscode-1.145.0.json")
    mapped_modes, report = map_adapter(root, directory / "adapter.json", theme, overrides)
    expected_target = {key: contract["target"][key] for key in ("application", "version", "os")}
    if report["target"] != expected_target or tuple(mapped_modes) != THEMES:
        raise TokenError("VS Code adapter must match the pinned target and generate Night, Day and Aurora")
    report = deepcopy(report)
    report["contractEvidence"] = contract
    report["capabilities"] = contract["capabilities"]
    report["contrast"] = {}
    report["substitutions"] = {}
    report["limitations"] = [
        "Checks measure emitted theme colours and sRGB alpha compositing, not every rendered pixel or extension-provided scope.",
        "ANSI black in Night/Aurora is an intentional palette slot, not ordinary text on the dark base; Day checks it as text.",
        "Terminal minimum contrast, SGR dim/truecolour/reverse video, font rendering and user overrides can change runtime colours.",
        "The theme configures no fonts, settings, key bindings, animations or executable extension entry points.",
        "Standard vs/vs-dark editor selections retain syntax: editor.selectionForeground applies only to high-contrast themes in the pinned renderer and is not emitted.",
    ]
    files = {}
    for mode, mapped in mapped_modes.items():
        validate_targets(mapped, contract)
        colors = {key.removeprefix("colors."): value for key, value in mapped.items() if key.startswith("colors.")}
        substitutions = []
        selected_text = {key: value for key, value in mapped.items() if key.startswith(("tm.", "semantic."))}
        selected_text["colors.editor.foreground"] = colors["editor.foreground"]
        original_selection = colors["editor.selectionBackground"]
        colors["editor.selectionBackground"], mixture = editor_selection_fill(original_selection, selected_text, mode)
        if colors["editor.selectionBackground"] != original_selection:
            substitutions.append({"target": "colors.editor.selectionBackground", "source": original_selection,
                                  "emitted": colors["editor.selectionBackground"], "mixFraction": mixture,
                                  "mixToward": "white" if mode == "day" else "black",
                                  "reason": "Normal editor themes retain syntax on selection; use an adapter-local fill that preserves 4.5:1 text contrast."})
        for target, alpha in TRANSLUCENT.items():
            original = colors[target]
            if target == "editor.inactiveSelectionBackground":
                while alpha and not all(min(pair(value, original + f"{alpha:02x}", colors["editor.background"])[:2]) >= 4.5
                                        for value in selected_text.values()):
                    alpha -= 1
            colors[target] = original + f"{alpha:02x}"
            substitutions.append({"target": "colors." + target, "source": original, "emitted": colors[target],
                                  "reason": "VS Code requires a translucent decoration overlay; occurrences remain identifiable through the mapped border."})
        ansi = {key: value for key, value in colors.items() if key.startswith("terminal.ansi")
                and not (key == "terminal.ansiBlack" and mode != "day")}
        terminal_text = dict(ansi, **{"terminal.foreground": colors["terminal.foreground"]})
        for target in ("terminal.findMatchBackground", "terminal.findMatchHighlightBackground"):
            original = colors[target]
            colors[target] = terminal_find_fill(original, terminal_text, colors["terminal.background"])
            substitutions.append({"target": "colors." + target, "source": original, "emitted": colors[target],
                                  "reason": "Constrain the translucent find fill for retained ANSI text; the separate semantic accent border remains visible."})
        token_colors = [{"name": "Fjord " + role, "scope": scopes,
                         "settings": {"foreground": mapped["tm." + role],
                                      "fontStyle": "italic" if role in ("comment", "keyword") else ""}}
                        for role, scopes in SCOPES.items()]
        semantic = {key.removeprefix("semantic."): {"foreground": value,
                    "italic": key in ("semantic.comment", "semantic.keyword")}
                    for key, value in mapped.items() if key.startswith("semantic.")}
        document = {"name": "Fjord " + mode.title(), "type": "light" if mode == "day" else "dark",
                    "semanticHighlighting": True, "colors": colors, "tokenColors": token_colors,
                    "semanticTokenColors": semantic}
        checks = []

        def check(foreground_target, foreground, background_target, background, underlay, minimum=4.5):
            unrounded, rounded, composited = pair(foreground, background, underlay)
            actual = min(unrounded, rounded)
            underlay_color = color_from_hex(underlay) if isinstance(underlay, str) else underlay
            item = {"foreground": foreground_target, "background": background_target,
                    "foregroundHex": foreground, "backgroundHex": background, "underlayHex": hex_color(underlay_color),
                    "underlayComponents": underlay_color["components"],
                    "compositedBackgroundHex": composited, "unroundedRatio": unrounded,
                    "hexRatio": rounded, "ratio": actual, "minimum": minimum, "pass": actual >= minimum}
            checks.append(item)
            if not item["pass"]:
                raise TokenError(f"VS Code {mode} {foreground_target} on {background_target}: {actual:.8f}:1 below {minimum}:1")

        editor_text = {key: value for key, value in mapped.items() if key.startswith(("tm.", "semantic."))}
        editor_text["colors.editor.foreground"] = colors["editor.foreground"]
        for background in ("editor.background", "editor.lineHighlightBackground"):
            for foreground, value in editor_text.items():
                check(foreground, value, "colors." + background, colors[background], colors["editor.background"])
            for foreground in ("editorLineNumber.foreground", "editorLineNumber.activeForeground"):
                check("colors." + foreground, colors[foreground], "colors." + background, colors[background], colors["editor.background"])
        for background in ("editor.selectionBackground", "editor.inactiveSelectionBackground"):
            for foreground, value in selected_text.items():
                check(foreground, value, "colors." + background, colors[background], colors["editor.background"])
        for background in ("terminal.selectionBackground", "terminal.inactiveSelectionBackground"):
            check("colors.terminal.selectionForeground", colors["terminal.selectionForeground"],
                  "colors." + background, colors[background], colors["terminal.background"])
        for background, foreground in (("editor.findMatchBackground", "editor.findMatchForeground"),
                                       ("editor.findMatchHighlightBackground", "editor.findMatchHighlightForeground")):
            check("colors." + foreground, colors[foreground], "colors." + background, colors[background], colors["editor.background"])
        for background in ("editor.selectionHighlightBackground", "editor.wordHighlightBackground",
                           "editor.wordHighlightStrongBackground", "editor.wordHighlightTextBackground"):
            for foreground, value in editor_text.items():
                check(foreground, value, "colors." + background, colors[background], colors["editor.background"])
        for foreground, value in terminal_text.items():
            check("colors." + foreground, value, "colors.terminal.background", colors["terminal.background"], colors["terminal.background"])
            for background in ("terminal.findMatchBackground", "terminal.findMatchHighlightBackground"):
                check("colors." + foreground, value, "colors." + background, colors[background], colors["terminal.background"])
        for operation in ("inserted", "removed"):
            line = "diffEditor." + operation + "LineBackground"
            text = "diffEditor." + operation + "TextBackground"
            line_underlay = composite(colors[line], colors["editor.background"])
            for foreground, value in editor_text.items():
                check(foreground, value, "colors." + line, colors[line], colors["editor.background"])
                check(foreground, value, "colors." + text, colors[text], line_underlay)
        for foreground, background in (("input.foreground", "input.background"), ("input.placeholderForeground", "input.background"),
                                      ("sideBar.foreground", "sideBar.background"), ("statusBar.foreground", "statusBar.background"),
                                      ("titleBar.activeForeground", "titleBar.activeBackground"), ("button.foreground", "button.background"),
                                      ("menu.foreground", "menu.background"), ("menu.selectionForeground", "menu.selectionBackground"),
                                      ("list.activeSelectionForeground", "list.activeSelectionBackground"),
                                      ("list.inactiveSelectionForeground", "list.inactiveSelectionBackground"),
                                      ("list.focusForeground", "list.focusBackground")):
            check("colors." + foreground, colors[foreground], "colors." + background, colors[background], colors[background])
        for background in ("editor.background", "editor.lineHighlightBackground", "sideBar.background", "panel.background"):
            check("colors.focusBorder", colors["focusBorder"], "colors." + background, colors[background], colors[background], 3)
        for border, background, underlay in (
            ("editor.selectionHighlightBorder", "editor.selectionHighlightBackground", "editor.background"),
            ("editor.wordHighlightBorder", "editor.wordHighlightBackground", "editor.background"),
            ("editor.wordHighlightStrongBorder", "editor.wordHighlightStrongBackground", "editor.background"),
            ("editor.wordHighlightTextBorder", "editor.wordHighlightTextBackground", "editor.background"),
            ("editor.findMatchBorder", "editor.findMatchBackground", "editor.background"),
            ("editor.findMatchHighlightBorder", "editor.findMatchHighlightBackground", "editor.background"),
            ("terminal.findMatchBorder", "terminal.findMatchBackground", "terminal.background"),
            ("terminal.findMatchHighlightBorder", "terminal.findMatchHighlightBackground", "terminal.background"),
        ):
            check("colors." + border, colors[border], "colors." + background, colors[background], colors[underlay], 3)
            check("colors." + border, colors[border], "colors." + underlay, colors[underlay], colors[underlay], 3)
        for foreground, background in (("editorCursor.foreground", "editor.background"),
                                      ("terminalCursor.foreground", "terminal.background"),
                                      ("scrollbarSlider.background", "editor.background"),
                                      ("scrollbarSlider.hoverBackground", "editor.background"),
                                      ("scrollbarSlider.activeBackground", "editor.background")):
            check("colors." + foreground, colors[foreground], "colors." + background, colors[background], colors[background], 3)
        for target in report["modes"][mode]["mapped"]:
            report["modes"][mode]["mapped"][target]["emittedValue"] = colors[target.removeprefix("colors.")] if target.startswith("colors.") else mapped[target]
        report["contrast"][mode] = checks
        report["substitutions"][mode] = substitutions
        report["modes"][mode]["selectionVisibility"] = {
            "backgroundHex": colors["editor.background"], "selectionHex": colors["editor.selectionBackground"],
            "ratio": min(pair(colors["editor.selectionBackground"], colors["editor.background"], colors["editor.background"])[:2]),
            "note": "Descriptive selection/base comparison; native visibility still requires inspection, not a WCAG pass criterion.",
        }
        files[f"themes/fjord-{mode}-color-theme.json"] = json_text(document)
    package = read_json(directory / "package.json")
    validate_package(package, files)
    files["package.json"] = json_text(package)
    files["report.json"] = json_text(report)
    files["README.md"] = (directory / "README.md").read_text(encoding="utf-8")
    files["LICENSE"] = (directory / "LICENSE").read_text(encoding="utf-8")
    return files


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--root", type=Path, default=ROOT, help="Fjord source project")
    parser.add_argument("--output-dir", type=Path, help="destination; default: dist/adapters/vscode")
    parser.add_argument("--theme", type=Path, help="replace the theme layer in all modes")
    parser.add_argument("--override", type=Path, action="append", default=[], help="override; repeat in precedence order")
    args = parser.parse_args(argv)
    try:
        files = build_bundle(args.root, args.theme, args.override)
        destination = args.output_dir or args.root / "dist/adapters/vscode"
        write_adapter_files(args.root, destination, files, args.override + ([args.theme] if args.theme else []))
        print(f"Built Fjord Night, Day and Aurora: {destination}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
