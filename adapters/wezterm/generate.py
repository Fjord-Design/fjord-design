"""Build the declarative reference adapter without changing WezTerm settings."""

import argparse
import json
from pathlib import Path
import re
import sys

from fjord_tokens.adapters import map_adapter, write_adapter_files
from fjord_tokens.dtcg import TokenError, hex_color
from fjord_tokens.project import Project
from fjord_tokens.code_roles import ROLES
from fjord_tokens.project import metadata


ROOT = Path(__file__).resolve().parents[2]


def scheme(mode, values):
    """Serialize the mapped fields using the established XER-116 format."""
    fields = ("foreground", "background", "cursor_bg", "cursor_border", "cursor_fg", "selection_bg", "selection_fg")
    targets = set(fields) | {f"{field}.{i}" for field in ("ansi", "brights") for i in range(8)}
    if set(values) != targets:
        raise TokenError(f"WezTerm mapping: missing targets {sorted(targets - values.keys())}; "
                         f"unknown targets {sorted(values.keys() - targets)}")
    if any(not isinstance(value, str) or not re.fullmatch(r"#[0-9a-fA-F]{6}", value) for value in values.values()):
        raise TokenError("WezTerm mapping: every target must resolve to an opaque sRGB HEX colour")
    header = f"# Fjord contract {metadata()['contractVersion']}; mode {mode}; sRGB HEX\n[colors]\n"
    lines = [f'{field} = "{values[field]}"\n' for field in fields]
    for field in ("ansi", "brights"):
        lines.append(field + " = [" + ", ".join('"' + values[f"{field}.{i}"] + '"' for i in range(8)) + "]\n")
    return header + "".join(lines) + f'\n[metadata]\nname = "Fjord {mode.title()}"\nauthor = "Fjord contributors"\n'


def bundle(root=ROOT, theme=None, overrides=()):
    modes, report = map_adapter(root, "adapters/wezterm/adapter.json", theme, overrides)
    files = {f"{mode}.toml": scheme(mode, values) for mode, values in modes.items()}
    files["report.json"] = json.dumps(report, ensure_ascii=False, indent=2, allow_nan=False) + "\n"
    directory = Path(root) / "adapters/wezterm"
    for filename in ("init.lua", "prompt.py", "README.md", "LICENSE"):
        files[filename] = (directory / filename).read_text()
    palette = {}
    for mode in modes:
        composition = Project(root).compose(mode, theme, overrides)
        palette[mode] = {name: hex_color(composition.values[role.token]) for name, role in ROLES.items() if name.startswith("prompt-")}
    files["prompt-colors.json"] = json.dumps(palette, indent=2) + "\n"
    return files


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--root", type=Path, default=ROOT, help="Fjord source directory")
    parser.add_argument("--theme", type=Path, help="replace the default theme layer in every mode")
    parser.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
    parser.add_argument("--output-dir", type=Path, help="destination; default: dist/adapters/wezterm")
    args = parser.parse_args(argv)
    try:
        files = bundle(args.root, args.theme, args.override)
        destination = args.output_dir or args.root / "dist/adapters/wezterm"
        inputs = [*args.override] + ([args.theme] if args.theme else [])
        write_adapter_files(args.root, destination, files, inputs)
        print(f"Built WezTerm adapter: {destination}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
