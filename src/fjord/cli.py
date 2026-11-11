"""Commands for validating and building Fjord DTCG tokens."""

import argparse
import json
from pathlib import Path
import sys
import tempfile

from .contract import MODES
from .dtcg import TokenError
from .project import Project, validate_project


ROOT = Path(__file__).resolve().parents[2]


def write_json_output(path, document, forbidden):
    write_text_output(path, json.dumps(document, ensure_ascii=False, indent=2, allow_nan=False) + "\n", forbidden)


def write_text_output(path, content, forbidden):
    write_binary_output(path, content.encode("utf-8"), forbidden)


def write_binary_output(path, content, forbidden):
    path = Path(path).resolve()
    if any(path == p.resolve() or p.is_dir() and path.is_relative_to(p.resolve()) for p in forbidden):
        raise TokenError(f"{path}: build output must not replace token sources")
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile("wb", dir=path.parent, delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(content)
        temporary.replace(path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def parser():
    result = argparse.ArgumentParser(description="Validate and export the Fjord design foundation.", allow_abbrev=False)
    commands = result.add_subparsers(dest="command", required=True)
    validate = commands.add_parser("validate", help="check tokens, contrast, fonts, icons and licences", allow_abbrev=False)
    validate.add_argument("--root", type=Path, default=ROOT, help="project directory")
    for name, help_text in (("build", "compose validated DTCG tokens for one mode"),
                            ("export", "export validated tokens as DTCG, resolved JSON or CSS")):
        command = commands.add_parser(name, help=help_text, allow_abbrev=False)
        command.add_argument("--root", type=Path, default=ROOT, help="project directory")
        command.add_argument("--mode", choices=MODES, default="night")
        command.add_argument("--theme", type=Path, help="replace the default theme layer")
        command.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
        command.add_argument("--output", type=Path, help="output path; defaults to dist/ inside the project")
        if name == "export":
            command.add_argument("--format", choices=("dtcg", "resolved", "css"), required=True)
    return result


def source_paths(root):
    return [root / name for name in ("tokens", "src", "docs", "assets", "examples", "tests", "scripts",
                                     ".github", ".git", "README.md", "LICENSE", "AGENTS.md", "CONTRIBUTING.md",
                                     "CHANGELOG.md", "mise.toml", ".gitignore")]


def main(argv=None):
    args = parser().parse_args(argv)
    try:
        report = validate_project(args.root)
        if args.command == "validate":
            for warning in report.deprecated:
                print(f"WARN deprecated {warning}", file=sys.stderr)
            from .fonts import load_fonts
            from .icons import load_icons

            fonts = load_fonts(args.root)
            icons = load_icons(args.root)
            faces = sum(len(family["faces"]) for family in fonts["families"])
            print(f"PASS: {report.tokens} DTCG tokens per mode; {len(MODES)} modes; "
                  f"{report.contrast_checks} contrast pairs in sRGB and HEX; "
                  f"{faces} font faces and {len(icons['icons'])} icons with pinned licences.")
            return 0

        composition = Project(args.root).compose(args.mode, args.theme, args.override)
        for warning in composition.deprecated:
            print(f"WARN deprecated {warning}", file=sys.stderr)
        forbidden = source_paths(args.root) + args.override + ([args.theme] if args.theme else [])
        from .export import export

        if args.command == "build":
            result = export(composition, "dtcg")
            destination = args.output or args.root / "dist" / f"{args.mode}.tokens.json"
        else:
            result = export(composition, args.format)
            suffix = {"dtcg": "dtcg.json", "resolved": "resolved.json", "css": "tokens.css"}[args.format]
            destination = args.output or args.root / "dist" / f"{args.mode}.{suffix}"
        if isinstance(result, dict):
            write_json_output(destination, result, forbidden)
        else:
            write_text_output(destination, result, forbidden)
        print(f"Wrote {args.mode}: {destination}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0
