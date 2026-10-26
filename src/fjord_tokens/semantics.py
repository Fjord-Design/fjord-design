"""Build semantic role/state evidence and the scoped theme reference styles."""

from html import escape
import json
from pathlib import Path

from .dtcg import TokenError, hex_color
from .fonts import family_css
from .foundations import css_value
from .palette import build_palette
from .project import Project


def variable(path):
    name = "button-primary-" + path.rsplit(".", 1)[1] if path.startswith("component.") else path.rsplit(".", 1)[1]
    return "--fjord-" + name


def semantic_report(root):
    root = Path(root)
    project = Project(root)
    identities, contract = {}, None
    for identity, theme in (("fjord", None), ("fern", root / "examples/themes/fern.tokens.json")):
        palette = build_palette(root, theme)
        modes = {}
        for mode in ("night", "day", "aurora"):
            composition = project.compose(mode, theme)
            bindings = {path: {"type": token["$type"], "alias": token["$value"]} for path, token in composition.tokens.items()
                        if path.startswith("component.")}
            if contract is None:
                contract = bindings
            elif contract != bindings:
                raise TokenError("Identity changes must preserve the component token contract")
            paths = {role["token"] for role in project.usage["roles"].values()}
            paths.update(path for role in project.usage["roles"].values() for path in role["backgrounds"])
            paths.update(path for path in composition.values if path.startswith(("component.", "semantic.shadow.")))
            modes[mode] = {"colors": palette["modes"][mode]["colors"], "pairs": palette["modes"][mode]["contrastPairs"],
                           "values": {path: {"type": composition.tokens[path]["$type"], "value": composition.values[path]}
                                      for path in sorted(paths)}}
        identities[identity] = modes
    return {"metadata": palette["metadata"], "usage": project.usage, "identities": identities,
            "componentContract": contract, "defaultPreference": "night", "systemMapping": {"light": "day", "dark": "night"}}


def declarations(data, mode):
    properties = []
    for path, item in data["values"].items():
        value = family_css(item["value"]) if item["type"] == "fontFamily" else css_value(item["value"], item["type"], path)
        properties.append(f"{variable(path)}:{value};")
    return "".join(properties) + ("color-scheme:light;" if mode == "day" else "color-scheme:dark;")


def semantic_css(report):
    blocks = []
    for identity, modes in report["identities"].items():
        for mode, data in modes.items():
            selector = f'[data-fjord-mode="{mode}"]'
            if identity == "fern":
                selector += '[data-fjord-identity="fern"]'
            elif mode == "night":
                selector = ":root," + selector
            blocks.append(selector + " {" + declarations(data, mode) + "}")
    system = []
    for identity, modes in report["identities"].items():
        selector = '[data-fjord-preference="system"][data-fjord-mode]'
        if identity == "fern":
            selector += '[data-fjord-identity="fern"]'
        system.append(selector + " {" + declarations(modes["day"], "day") + "}")
    blocks.append("@media (prefers-color-scheme:light) {" + "".join(system) + "}")
    blocks.append('''
.fjord-foundation.fjord-semantic {background:var(--fjord-bg-base);color:var(--fjord-ink);}
.fjord-semantic ::selection {background:var(--fjord-selection);color:var(--fjord-ink);}
.fjord-semantic a {color:var(--fjord-accent);text-decoration:underline;text-underline-offset:.2em;}
.fjord-semantic a:hover {color:var(--fjord-accent-hover);text-decoration-thickness:2px;}
.fjord-foundation.fjord-semantic .semantic-primary {background:var(--fjord-button-primary-background);color:var(--fjord-button-primary-foreground);
 border-color:var(--fjord-button-primary-background);border-radius:var(--fjord-button-primary-radius);font-family:var(--fjord-button-primary-font-family);font-size:var(--fjord-button-primary-font-size);}
.fjord-foundation.fjord-semantic .semantic-primary:hover,.fjord-foundation.fjord-semantic .semantic-primary[data-sample-state="hover"] {background:var(--fjord-button-primary-hover-background);border-color:var(--fjord-button-primary-hover-background);}
.fjord-foundation.fjord-semantic .semantic-primary:active,.fjord-foundation.fjord-semantic .semantic-primary[data-sample-state="pressed"] {background:var(--fjord-button-primary-pressed-background);border-color:var(--fjord-button-primary-pressed-background);box-shadow:var(--fjord-shadow-pressed);}
.fjord-semantic [data-sample-state="focus-visible"] {outline:var(--fjord-border-focus) solid var(--fjord-focus-ring);outline-offset:var(--fjord-focus-offset);}
.fjord-foundation.fjord-semantic .semantic-choice[aria-pressed="true"] {background:var(--fjord-accent-soft);color:var(--fjord-accent-ink);border:var(--fjord-border-active) solid var(--fjord-accent);}
.fjord-foundation.fjord-semantic .fjord-control:disabled,.fjord-foundation.fjord-semantic .semantic-input:disabled {background:var(--fjord-bg-surface);color:var(--fjord-ink-disabled);border-color:var(--fjord-line);cursor:not-allowed;box-shadow:none;}
.fjord-semantic .semantic-input {font:inherit;line-height:1.6;min-block-size:var(--fjord-control-comfortable);padding:var(--fjord-space-2);max-inline-size:100%;inline-size:100%;border:var(--fjord-border-hairline) solid var(--fjord-line-strong);border-radius:var(--fjord-radius-control);background:var(--fjord-bg-surface);color:var(--fjord-ink);}
.fjord-semantic .semantic-input[aria-invalid="true"] {border-color:var(--fjord-danger);}
.fjord-semantic .semantic-error {color:var(--fjord-danger);}
.fjord-semantic .semantic-help {color:var(--fjord-ink-muted);}
.fjord-semantic .semantic-status {padding:var(--fjord-space-3);border-radius:var(--fjord-radius-panel);}
.fjord-semantic .semantic-status[data-status="info"] {background:var(--fjord-info-soft);color:var(--fjord-info);}
.fjord-semantic .semantic-status[data-status="success"] {background:var(--fjord-success-soft);color:var(--fjord-success);}
.fjord-semantic .semantic-status[data-status="warning"] {background:var(--fjord-warning-soft);color:var(--fjord-warning);}
.fjord-semantic .semantic-status[data-status="danger"] {background:var(--fjord-danger-soft);color:var(--fjord-danger);}
.fjord-semantic dialog {max-inline-size:min(32rem,calc(100% - 2rem));padding:var(--fjord-space-6);background:var(--fjord-bg-raised);color:var(--fjord-ink);border:1px solid var(--fjord-line);border-radius:var(--fjord-radius-window);box-shadow:var(--fjord-shadow-overlay);}
.fjord-semantic dialog::backdrop {background:var(--fjord-scrim);}
@media (forced-colors:active) {
 .fjord-foundation.fjord-semantic {background:Canvas;color:CanvasText;}
 .fjord-semantic a {color:LinkText;}
 .fjord-foundation.fjord-semantic .semantic-primary {background:ButtonFace;color:ButtonText;border-color:ButtonText;}
 .fjord-foundation.fjord-semantic .semantic-choice[aria-pressed="true"] {background:Highlight;color:HighlightText;border-color:Highlight;}
 .fjord-foundation.fjord-semantic .fjord-control:disabled,.fjord-foundation.fjord-semantic .semantic-input:disabled {color:GrayText;border-color:GrayText;}
 .fjord-semantic .semantic-help,.fjord-semantic .semantic-error {color:CanvasText;}
 .fjord-semantic .semantic-input,.fjord-semantic .semantic-status,.fjord-semantic dialog {background:Canvas;color:CanvasText;border:1px solid CanvasText;box-shadow:none;}
 .fjord-semantic [data-sample-state="focus-visible"] {outline-color:Highlight;}
}
''')
    return "\n".join(blocks) + "\n"


def render_semantics(report):
    template = Path(__file__).with_name("templates") / "semantics.html"
    rows = []
    for name, role in report["usage"]["roles"].items():
        cells = [escape(name), escape(role["token"])]
        for mode in ("night", "day", "aurora"):
            value = report["identities"]["fjord"][mode]["values"][role["token"]]["value"]
            cells.append(escape(hex_color(value)))
        cells.append(escape(role["description"]))
        rows.append("<tr>" + "".join(f"<td>{cell}</td>" for cell in cells) + "</tr>")
    pairs = []
    for identity, modes in report["identities"].items():
        for mode, data in modes.items():
            for pair in data["pairs"]:
                pairs.append(f'<tr data-pair-identity="{identity}" data-pair-mode="{mode}"><td>{escape(pair["foreground"])}</td>'
                             + f'<td>{escape(pair["background"])}</td><td>{pair["minimum"]:g}:1</td><td>{pair["hexRatio"]:.4f}:1</td></tr>')
    return template.read_text(encoding="utf-8").replace("{{ROLES}}", "".join(rows)).replace("{{PAIRS}}", "".join(pairs))


def semantic_bundle(root):
    report = semantic_report(root)
    return {"semantics.css": semantic_css(report).encode(), "semantics.html": render_semantics(report).encode(),
            "semantics.json": (json.dumps(report, ensure_ascii=False, indent=2) + "\n").encode(),
            "theme-preference.js": (Path(root) / "reference/svelte/src/lib/theme-preference.js").read_bytes()}
