"""Render typography roles, font fallbacks and code feature controls."""

from html import escape
from pathlib import Path

from .contract import THEMES
from .dtcg import EXTENSION, hex_color
from .fonts import ROLES
from .project import Project, validate_project


DANISH = "Ærø, øl og blåbær. ÆØÅ æøå. Café, 0123456789. Il1 | O0o."


def typography_report(root, manifest):
    validate_project(root)
    composition = Project(root).compose("night")
    roles = {}
    for role in ROLES:
        path = "semantic.typography." + role
        roles[role] = {"value": composition.values[path],
                       "style": composition.tokens[path]["$extensions"][EXTENSION]["fontStyle"]}
    return {"roles": roles, "fonts": manifest, "ligaturesDefault": True,
            "cursive": {"default": False, "family": "Cascadia Code", "feature": "ss01", "mechanism": "font-specific CSS styleset"},
            "glyphProbe": DANISH}


def render_typography(root, report):
    css, project = [], Project(root)
    for mode in THEMES:
        values = project.compose(mode).values
        names = "bg-base bg-surface bg-sunken ink ink-muted accent on-accent focus-ring line line-strong syn-comment syn-keyword syn-string".split()
        css.append(f'html[data-theme="{mode}"] {{' + "".join(f'--{name}:{hex_color(values["semantic.color." + name])};' for name in names) + "}")
    rows = []
    for role, data in report["roles"].items():
        value = data["value"]
        rows.append('<article class="type-row">'
                    + f'<div class="role-meta"><h3><code>{role}</code></h3>'
                    + f'<span>{escape(value["fontFamily"][0])} · {value["fontSize"]["value"]:g}{value["fontSize"]["unit"]}'
                    + f' / {value["lineHeight"]:g} · {value["fontWeight"]} · {data["style"]}</span>'
                    + f'<span>Letter spacing: {value["letterSpacing"]["value"]:g}{value["letterSpacing"]["unit"]}</span></div>'
                    + f'<p class="sample fjord-{role}">{escape(DANISH)}</p></article>')
    template = Path(__file__).with_name("templates") / "typography.html"
    html = template.read_text(encoding="utf-8")
    return html.replace("{{CSS}}", "\n".join(css)).replace("{{ROLES}}", "\n".join(rows))
