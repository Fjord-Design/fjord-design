"""Generate the layout foundation stylesheet and an interactive specimen."""

from html import escape
import json
from pathlib import Path

from .contract import THEMES
from .dtcg import EXTENSION, TokenError, hex_color, read_json
from .fonts import rem
from .icons import icon_module, load_icons, svg_icon
from .project import Project, validate_project


GROUPS = ("spacing", "radius", "border-width", "duration", "easing", "z-index", "size", "breakpoint", "layout", "density", "shadow")
COLORS = "bg-base bg-surface bg-sunken bg-raised bg-hover ink ink-muted accent accent-hover on-accent focus-ring line line-strong success warning danger".split()


def css_value(value, kind, path=""):
    if kind == "color":
        return hex_color(value)
    if kind == "dimension":
        if path.startswith("semantic.breakpoint."):
            return f"{value['value']:g}{value['unit']}"
        return rem(value)
    if kind == "duration":
        return f"{value['value']:g}{value['unit']}"
    if kind == "cubicBezier":
        return "cubic-bezier(" + ",".join(f"{item:g}" for item in value) + ")"
    if kind == "shadow":
        return ",".join(("inset " if item.get("inset") else "")
                        + " ".join(rem(item[name]) for name in ("offsetX", "offsetY", "blur", "spread"))
                        + " " + hex_color(item["color"]) for item in value)
    return str(value)


def foundation_report(root):
    validate_project(root)
    project = Project(root)
    base = read_json(project.source(project.contract["base"]))
    if base["$extensions"][EXTENSION].get("iconAssets") != "assets/icons/manifest.json":
        raise TokenError("iconAssets: reference assets/icons/manifest.json")
    manifest, bodies = load_icons(root)
    expected = icon_module(bodies)
    generated = Path(root) / "reference/svelte/src/lib/icons.generated.ts"
    if generated.read_text(encoding="utf-8") != expected:
        raise TokenError("Icon data differs from the pinned SVGs; run mise run icons:sync")
    modes = {}
    for mode in THEMES:
        composition = project.compose(mode)
        modes[mode] = {path: {"type": composition.tokens[path]["$type"], "value": value}
                       for path, value in composition.values.items()
                       if any(path.startswith(f"semantic.{group}.") for group in GROUPS)
                       or path in {"semantic.color." + name for name in COLORS}}
    return {"contractVersion": project.contract["version"], "modes": modes, "icons": manifest,
            "densityDefault": "comfortable", "reducedMotion": "No transitions, animation or smooth scrolling",
            "forcedColors": "System colours, visible borders and focus outlines"}, bodies


def foundation_css(report):
    blocks = []
    for mode, tokens in report["modes"].items():
        selectors = (":root," if mode == "night" else "") + f'[data-fjord-mode="{mode}"]'
        blocks.append(selectors + " {" + "".join(
            f"--fjord-{path.rsplit('.', 1)[1]}:{css_value(item['value'], item['type'], path)};" for path, item in tokens.items()) + "}")
    widths = {name: report["modes"]["night"]["semantic.breakpoint.breakpoint-" + name]["value"] for name in ("sm", "md", "lg")}
    blocks.append('''
.fjord-foundation {
  --fjord-gap:var(--fjord-gap-comfortable);--fjord-padding:var(--fjord-padding-comfortable);
  --fjord-control-min:var(--fjord-control-comfortable);--fjord-button-padding:var(--fjord-space-2);
  background:var(--fjord-bg-base);color:var(--fjord-ink);font-synthesis:none;
}
.fjord-foundation[data-fjord-density="compact"] {
  --fjord-gap:var(--fjord-gap-compact);--fjord-padding:var(--fjord-padding-compact);
  --fjord-control-min:var(--fjord-control-compact);--fjord-button-padding:var(--fjord-space-1);
}
.fjord-foundation *, .fjord-foundation *::before, .fjord-foundation *::after {box-sizing:border-box;}
.fjord-foundation .fjord-page {max-inline-size:var(--fjord-content-max);margin-inline:auto;padding:var(--fjord-page-gutter);overflow-wrap:anywhere;}
.fjord-foundation .fjord-grid {display:grid;grid-template-columns:minmax(0,1fr);gap:var(--fjord-gap);}
.fjord-foundation .fjord-stack {display:grid;gap:var(--fjord-gap);}
.fjord-foundation .fjord-panel {min-inline-size:0;padding:var(--fjord-padding);border-radius:var(--fjord-radius-panel);background:var(--fjord-bg-surface);}
.fjord-foundation .fjord-window {border-radius:var(--fjord-radius-window);background:var(--fjord-bg-raised);box-shadow:var(--fjord-shadow-overlay);}
.fjord-foundation .fjord-separator {border:0;border-block-start:var(--fjord-border-hairline) solid var(--fjord-line);margin-block:var(--fjord-space-6);}
.fjord-foundation .fjord-control {display:inline-flex;align-items:center;justify-content:center;gap:var(--fjord-space-2);font:inherit;line-height:1.5;text-align:start;white-space:normal;overflow-wrap:anywhere;
  min-block-size:var(--fjord-control-min);min-inline-size:var(--fjord-control-min);max-inline-size:100%;padding:var(--fjord-button-padding) var(--fjord-space-4);
  border:var(--fjord-border-hairline) solid var(--fjord-line-strong);border-radius:var(--fjord-radius-full);background:var(--fjord-bg-surface);color:var(--fjord-ink);cursor:pointer;
  transition:box-shadow var(--fjord-duration-fast) var(--fjord-easing-standard);
}
.fjord-foundation .fjord-control:hover {background:var(--fjord-bg-hover);}
.fjord-foundation .fjord-control[data-primary] {background:var(--fjord-accent);color:var(--fjord-on-accent);border-color:var(--fjord-accent);}
.fjord-foundation .fjord-control[data-primary]:hover {background:var(--fjord-accent-hover);}
.fjord-foundation :focus-visible {outline:var(--fjord-border-focus) solid var(--fjord-focus-ring);outline-offset:var(--fjord-focus-offset);}
.fjord-foundation .fjord-icon {display:inline-block;inline-size:var(--fjord-icon-default);block-size:var(--fjord-icon-default);flex-shrink:0;vertical-align:-.125em;}
.fjord-foundation .fjord-control .fjord-icon {inline-size:var(--fjord-icon-control);block-size:var(--fjord-icon-control);}
.fjord-foundation .fjord-status .fjord-icon {inline-size:var(--fjord-icon-status);block-size:var(--fjord-icon-status);}
.fjord-foundation .fjord-icon-button {padding:var(--fjord-button-padding);}
.fjord-foundation .fjord-reading {max-inline-size:70ch;}
.fjord-foundation .fjord-loading {display:inline-flex;align-items:center;gap:var(--fjord-space-2);}
.fjord-foundation .fjord-loading .fjord-icon {animation:fjord-spin calc(var(--fjord-duration-slow) * 4) linear infinite;}
.fjord-foundation .fjord-reveal {animation:fjord-enter var(--fjord-duration-base) var(--fjord-easing-standard);}
@media (max-width:480px) {
  .fjord-foundation .fjord-page {padding-inline:min(var(--fjord-page-gutter),4vw);}
  .fjord-foundation .fjord-panel {padding-inline:min(var(--fjord-padding),5vw);}
}
@keyframes fjord-spin {to {transform:rotate(360deg);}}
@keyframes fjord-enter {from {transform:translateY(var(--fjord-space-1));} to {transform:translateY(0);}}
@media (any-pointer:coarse) {
  .fjord-foundation .fjord-control {min-block-size:var(--fjord-target-touch);min-inline-size:var(--fjord-target-touch);}
}
@media (prefers-reduced-motion:reduce) {
  .fjord-foundation *, .fjord-foundation *::before, .fjord-foundation *::after {animation:none !important;transition:none !important;scroll-behavior:auto !important;}
}
@media (forced-colors:active) {
  .fjord-foundation {background:Canvas;color:CanvasText;}
  .fjord-foundation .fjord-panel,.fjord-foundation .fjord-window {background:Canvas;color:CanvasText;border:1px solid CanvasText;box-shadow:none;}
  .fjord-foundation .fjord-control {background:ButtonFace;color:ButtonText;border:1px solid ButtonText;}
  .fjord-foundation .fjord-control[data-primary] {background:Highlight;color:HighlightText;border-color:Highlight;}
  .fjord-foundation .fjord-separator {border-color:CanvasText;}
  .fjord-foundation :focus-visible {outline-color:Highlight;}
  .fjord-foundation .fjord-status {color:CanvasText;}
}
''')
    for name, columns in (("sm", 2), ("md", 3), ("lg", 3)):
        threshold = f"{widths[name]['value']:g}{widths[name]['unit']}"
        extra = "padding-inline:var(--fjord-page-gutter-wide);" if name == "sm" else ""
        blocks.append(f"@media (min-width:{threshold}) {{.fjord-foundation .fjord-grid {{grid-template-columns:repeat({columns},minmax(0,1fr));}}"
                      + (f".fjord-foundation .fjord-page {{{extra}}}" if extra else "") + "}")
    return "\n".join(blocks) + "\n"


def foundation_html(root, report, bodies):
    source = Path(__file__).with_name("templates") / "foundations.html"
    html = source.read_text(encoding="utf-8")
    rows = []
    values = report["modes"]["night"]
    for path, item in values.items():
        if path.startswith(("semantic.spacing.", "semantic.size.", "semantic.breakpoint.", "semantic.z-index.")):
            rows.append(f'<tr><th scope="row">{escape(path)}</th><td>{escape(css_value(item["value"], item["type"], path))}</td></tr>')
    icons = "".join(f'<li>{svg_icon(bodies, name)}<span>{escape(name)}</span></li>' for name in bodies)
    replacements = {"{{ICONS}}": icons, "{{TOKENS}}": "".join(rows)}
    for name in ("info", "success", "warning", "danger", "loading", "close", "add"):
        replacements["{{ICON_" + name.upper() + "}}"] = svg_icon(bodies, name)
    replacements["{{MEANINGFUL_ICON}}"] = svg_icon(bodies, "info", "Information")
    for marker, value in replacements.items():
        html = html.replace(marker, value)
    return html


def foundation_bundle(root):
    report, bodies = foundation_report(root)
    files = {"foundations.css": foundation_css(report).encode(), "foundations.html": foundation_html(root, report, bodies).encode(),
             "foundations.json": (json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode()}
    directory = Path(root) / "assets/icons"
    for name in ("manifest.json", "LICENSE", *(item["file"] for item in report["icons"]["icons"])):
        files["icons/" + name] = (directory / name).read_bytes()
    return files
