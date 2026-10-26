"""Commands for validating and building Fjord DTCG tokens."""

import argparse
import json
from pathlib import Path
import sys
import tempfile

from .contract import THEMES
from .dtcg import TokenError, nest
from .project import Project, check_contrast, metadata, validate_project


ROOT = Path(__file__).resolve().parents[2]


def write_output(path, document, forbidden):
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
    result = argparse.ArgumentParser(description="Validate and build Fjord DTCG tokens.", allow_abbrev=False)
    commands = result.add_subparsers(dest="command", required=True)
    validate = commands.add_parser("validate", help="check every mode and the platform tables", allow_abbrev=False)
    validate.add_argument("--root", type=Path, default=ROOT, help="project directory")
    build = commands.add_parser("build", help="compose validated DTCG tokens for one mode", allow_abbrev=False)
    build.add_argument("--root", type=Path, default=ROOT, help="project directory")
    build.add_argument("--mode", choices=THEMES, default="night")
    build.add_argument("--theme", type=Path, help="replace the default theme layer")
    build.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
    build.add_argument("--output", type=Path, help="output path; default: dist/<mode>.tokens.json")
    export = commands.add_parser("export", help="export validated tokens or a WezTerm colour scheme", allow_abbrev=False)
    export.add_argument("--root", type=Path, default=ROOT, help="project directory")
    export.add_argument("--mode", choices=THEMES, default="night")
    export.add_argument("--format", choices=("dtcg", "resolved", "css", "wezterm"), required=True)
    export.add_argument("--theme", type=Path, help="replace the default theme layer")
    export.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
    export.add_argument("--output", type=Path, help="output path; default: dist/<mode>.<format>")
    preview = commands.add_parser("preview", help="build the interactive code and terminal specimen", allow_abbrev=False)
    preview.add_argument("--root", type=Path, default=ROOT, help="project directory")
    preview.add_argument("--theme", type=Path, help="replace the default theme layer in every mode")
    preview.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
    preview.add_argument("--output", type=Path, help="HTML output path; default: dist/code-and-terminal.html")
    palette = commands.add_parser("palette", help="build colour scales and HEX contrast evidence", allow_abbrev=False)
    palette.add_argument("--root", type=Path, default=ROOT, help="project directory")
    palette.add_argument("--format", choices=("html", "json"), default="html")
    palette.add_argument("--theme", type=Path, help="replace the default theme layer in every mode")
    palette.add_argument("--override", type=Path, action="append", default=[], help="override file; repeat in precedence order")
    palette.add_argument("--output", type=Path, help="output path; default: dist/palette.<format>")
    typography = commands.add_parser("typography", help="validate or build self-hosted fonts and type scales", allow_abbrev=False)
    typography.add_argument("--root", type=Path, default=ROOT, help="project directory")
    typography.add_argument("--check", action="store_true", help="validate assets and roles without writing output")
    foundations = commands.add_parser("foundations", help="validate or build layout rules and icon assets", allow_abbrev=False)
    foundations.add_argument("--root", type=Path, default=ROOT, help="project directory")
    foundations.add_argument("--check", action="store_true", help="validate layout and icon assets without writing output")
    semantics = commands.add_parser("semantics", help="validate or build semantic roles, states and theme identity evidence", allow_abbrev=False)
    semantics.add_argument("--root", type=Path, default=ROOT, help="project directory")
    semantics.add_argument("--check", action="store_true", help="validate both identities in all modes without writing output")
    return result


def main(argv=None):
    args = parser().parse_args(argv)
    try:
        if args.command == "validate":
            report = validate_project(args.root)
            for warning in report.deprecated:
                print(f"WARN deprecated {warning}", file=sys.stderr)
            print(f"PASS: {report.tokens} DTCG tokens per mode; 3 modes; "
                  f"{report.contrast_checks} contrast pairs in sRGB and HEX; {report.platform_values} platform values.")
        elif args.command == "build":
            validate_project(args.root)
            project = Project(args.root)
            composition = project.compose(args.mode, args.theme, args.override)
            check_contrast(composition)
            info = dict(metadata(), mode=args.mode)
            destination = args.output or args.root / "dist" / f"{args.mode}.tokens.json"
            forbidden = [args.root / "tokens", *args.override] + ([args.theme] if args.theme else [])
            write_output(destination, nest(composition.tokens, info), forbidden)
            print(f"Built {args.mode}: {destination}")
        elif args.command == "export":
            from .export import export

            validate_project(args.root)
            composition = Project(args.root).compose(args.mode, args.theme, args.override)
            result = export(composition, args.format)
            suffix = {"dtcg": "dtcg.json", "resolved": "resolved.json", "css": "tokens.css", "wezterm": "wezterm.toml"}[args.format]
            destination = args.output or args.root / "dist" / f"{args.mode}.{suffix}"
            forbidden = [args.root / name for name in ("tokens", "src", "docs", "assets", "examples", "tests", "reference",
                                                      "scripts", "patches", ".github", ".git", "README.md", "LICENSE",
                                                      "mise.toml", "package.json", "bun.lock")]
            forbidden += [*args.override] + ([args.theme] if args.theme else [])
            if isinstance(result, dict):
                write_output(destination, result, forbidden)
            else:
                write_text_output(destination, result, forbidden)
            print(f"Exported {args.mode} {args.format}: {destination}")
        elif args.command == "preview":
            from .preview import render_preview
            from .fonts import font_bundle, load_fonts

            content = render_preview(args.root, args.theme, args.override)
            files = font_bundle(args.root, load_fonts(args.root))
            destination = args.output or args.root / "dist/code-and-terminal.html"
            forbidden = [args.root / name for name in ("tokens", "examples", "src", "assets", "docs", "tests", "reference",
                                                       "scripts", "patches", ".git", ".github", "README.md", "LICENSE", "mise.toml", "package.json", "bun.lock")]
            forbidden += args.override
            if args.theme:
                forbidden.append(args.theme)
            write_text_output(destination, content, forbidden)
            for name, data in files.items():
                write_binary_output(destination.parent / "fonts" / name, data, forbidden)
            print(f"Built code and terminal preview: {destination}")
        elif args.command == "palette":
            from .palette import build_palette, render_palette

            report = build_palette(args.root, args.theme, args.override)
            destination = args.output or args.root / f"dist/palette.{args.format}"
            forbidden = [args.root / name for name in ("tokens", "src", "docs", "assets", "examples", "tests", "reference",
                                                      "scripts", "patches", ".github", ".git", "README.md", "LICENSE",
                                                      "mise.toml", "package.json", "bun.lock")]
            forbidden += [*args.override] + ([args.theme] if args.theme else [])
            if args.format == "html":
                write_text_output(destination, render_palette(report), forbidden)
            else:
                write_output(destination, report, forbidden)
            print(f"Built colour palette: {destination}")
        elif args.command == "semantics":
            from .semantics import semantic_bundle

            files = semantic_bundle(args.root)
            if args.check:
                print("PASS: 34 semantic UI roles, 8 states and 2 identities across 3 modes.")
            else:
                forbidden = [args.root / name for name in ("assets", "tokens", "src", "docs", "tests", "reference", "scripts", "patches", ".git",
                                                           ".github", "examples", "README.md", "LICENSE", "mise.toml", "package.json", "bun.lock")]
                for name, data in files.items():
                    write_binary_output(args.root / "dist" / name, data, forbidden)
                print(f"Built semantic states: {args.root / 'dist/semantics.html'}")
        elif args.command == "foundations":
            from .foundations import foundation_bundle

            files = foundation_bundle(args.root)
            if args.check:
                print("PASS: layout, density, layers, motion and 16 pinned Phosphor Fill icons.")
            else:
                forbidden = [args.root / name for name in ("assets", "tokens", "src", "docs", "tests", "reference", "scripts", "patches", ".git",
                                                           ".github", "examples", "README.md", "LICENSE", "mise.toml", "package.json", "bun.lock")]
                for name, data in files.items():
                    write_binary_output(args.root / "dist" / name, data, forbidden)
                print(f"Built foundations: {args.root / 'dist/foundations.html'}")
        elif args.command == "typography":
            from .fonts import font_bundle, load_fonts
            from .typography import render_typography, typography_report

            manifest = load_fonts(args.root)
            report = typography_report(args.root, manifest)
            if args.check:
                print("PASS: 9 pinned WOFF2 faces, 5 licences and 15 typography roles.")
            else:
                files = font_bundle(args.root, manifest)
                html = render_typography(args.root, report)
                forbidden = [args.root / name for name in ("assets", "tokens", "src", "docs", "tests", "reference", "scripts", "patches", ".git",
                                                           ".github", "examples", "README.md", "LICENSE", "mise.toml", "package.json", "bun.lock")]
                for name, content in files.items():
                    write_binary_output(args.root / "dist/fonts" / name, content, forbidden)
                write_text_output(args.root / "dist/typography.html", html, forbidden)
                write_output(args.root / "dist/typography.json", report, forbidden)
                print(f"Built typography: {args.root / 'dist/typography.html'}")
    except (TokenError, OSError, UnicodeError) as error:
        print(f"ERROR {error}", file=sys.stderr)
        return 1
    return 0
