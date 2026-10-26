"""Build an offline code specimen from validated token values and annotated examples."""

from html import escape
from pathlib import Path
import re

from .code_roles import ROLES, syntax_distances
from .contract import ANSI, THEMES
from .dtcg import TokenError, hex_color, read_json
from .project import Project, check_contrast, contrast_value, validate_project


STATES = {"added": "diff-added-background", "changed": "diff-changed-background", "deleted": "diff-deleted-background"}
SAMPLES = {"typescript", "json", "shell", "terminal", "diff", "diagnostics", "git"}


def load_samples(root):
    path = Path(root) / "examples/code/samples.json"
    samples = read_json(path)
    if not isinstance(samples, list) or len(samples) != len(SAMPLES):
        raise TokenError(f"{path}: supply one example for each of {', '.join(sorted(SAMPLES))}")
    seen = set()
    for sample in samples:
        if not isinstance(sample, dict) or set(sample) != {"id", "title", "caption", "lines"}:
            raise TokenError(f"{path}: each example requires id, title, caption and lines")
        identifier = sample["id"]
        if not isinstance(identifier, str) or identifier not in SAMPLES or identifier in seen:
            raise TokenError(f"{path}: unknown or repeated example {identifier!r}")
        seen.add(identifier)
        if any(not isinstance(sample[key], str) or not sample[key].strip() for key in ("title", "caption")):
            raise TokenError(f"{path}:{identifier}: title and caption must be non-empty text")
        if not isinstance(sample["lines"], list) or not sample["lines"]:
            raise TokenError(f"{path}:{identifier}: lines must be a non-empty array")
        for line in sample["lines"]:
            if not isinstance(line, dict) or set(line) - {"spans", "state"} or "spans" not in line:
                raise TokenError(f"{path}:{identifier}: a line requires spans and an optional diff state")
            if "state" in line and (not isinstance(line["state"], str) or line["state"] not in STATES):
                raise TokenError(f"{path}:{identifier}: unknown diff state")
            if not isinstance(line["spans"], list) or not line["spans"]:
                raise TokenError(f"{path}:{identifier}: spans must be a non-empty array")
            for span in line["spans"]:
                if not isinstance(span, list) or len(span) != 2 or not all(isinstance(v, str) for v in span):
                    raise TokenError(f"{path}:{identifier}: spans contain a role name and text")
                if span[0] not in ROLES or not ROLES[span[0]].backgrounds:
                    raise TokenError(f"{path}:{identifier}: unknown or non-text role {span[0]!r}")
                if "\n" in span[1] or "\r" in span[1]:
                    raise TokenError(f"{path}:{identifier}: use a separate line for each line of text")
    return samples


def check_samples(samples, composition):
    """Check the foreground and actual surface of every rendered example span."""
    values = composition.values
    count = 0
    for sample in samples:
        for number, line in enumerate(sample["lines"], 1):
            background = ROLES[STATES[line["state"]]].token if "state" in line else "semantic.color.bg-base"
            foregrounds = list(line["spans"])
            if "state" in line:
                foregrounds.append([f"diff-{line['state']}", "marker"])
            for role_name, _ in foregrounds:
                role = ROLES[role_name]
                ratio = contrast_value(values[role.token], values[background])
                minimum = 4.5
                if role_name == "syntax-comment" and background == "semantic.color.bg-base":
                    minimum = 7.5 if composition.mode == "night" else 5
                if ratio < minimum:
                    raise TokenError(f"{sample['id']}:{number} {role_name} [{composition.mode}]: "
                                     f"{ratio:.6f}:1 is below {minimum:g}:1 on {background}")
                count += 1
    return count


def render_samples(samples):
    panels = []
    for sample in samples:
        lines = []
        numbered = sample["id"] in {"typescript", "json", "shell"}
        for number, line in enumerate(sample["lines"], 1):
            spans = []
            state = line.get("state", "")
            if state:
                spans.append(f'<span class="change-marker r-diff-{state}">{escape(ROLES[f"diff-{state}"].marker)}</span>')
            for name, text in line["spans"]:
                role = ROLES[name]
                marker = f'<span class="status-marker">{escape(role.marker)}</span> ' if role.marker else ""
                spans.append(f'<span class="r-{name}">{marker}{escape(text)}</span>')
            label = f'<span class="line-number" aria-hidden="true">{number}</span>' if numbered else ""
            lines.append(f'<div class="code-line state-{state}">{label}<code>{"".join(spans)}</code></div>')
        identifier = sample["id"]
        panels.append(f'<section class="sample sample-{identifier}" aria-labelledby="heading-{identifier}">'
                      f'<header><h2 id="heading-{identifier}">{escape(sample["title"])}</h2>'
                      f'<p>{escape(sample["caption"])}</p></header>'
                      f'<div class="code-lines" tabindex="0" role="region" aria-label="{escape(sample["title"])} example">'
                      + "".join(lines) + '</div>'
                      f'<p class="scroll-hint" id="scroll-hint-{identifier}" hidden>Scroll horizontally to see the full example.</p>'
                      '</section>')
    return "\n".join(panels)


def render_preview(root, theme=None, overrides=()):
    validate_project(root)
    samples = load_samples(root)
    project = Project(root)
    theme_styles = []
    metrics = []
    for mode in THEMES:
        composition = project.compose(mode, theme, overrides)
        check_contrast(composition)
        check_samples(samples, composition)
        properties = [f"--{path.removeprefix('semantic.color.')}: {hex_color(value)};"
                      for path, value in composition.values.items() if path.startswith("semantic.color.")]
        theme_styles.append(f'html[data-theme="{mode}"] {{' + "".join(properties) + "}")
        comment = contrast_value(composition.values["semantic.color.syn-comment"], composition.values["semantic.color.bg-base"])
        hint = contrast_value(composition.values["semantic.color.ansi-bright-black"], composition.values["semantic.color.bg-base"])
        separation = min(distance for _, _, distance, minimum in syntax_distances(composition) if minimum == 7)
        metrics.append(f'<p class="mode-metric metric-{mode}">Comments {comment:.2f}:1 · '
                       f'Terminal hints {hint:.2f}:1 · Syntax distance {separation:.2f}</p>')
    role_styles = []
    for name, role in ROLES.items():
        declaration = f"color: var(--{role.color});"
        if role.emphasis == "italic":
            declaration += "font-style: italic;font-variant-alternates:var(--fjord-code-alternates,normal);"
        elif role.emphasis:
            declaration += f"text-decoration: underline {role.emphasis}; text-underline-offset: .3em;"
        role_styles.append(f".r-{name} {{{declaration}}}")
    swatches = []
    for index, name in enumerate(ANSI):
        label = name.removeprefix("ansi-").replace("-", " ")
        swatches.append(f'<div class="ansi-swatch"><span style="background:var(--{name})" aria-hidden="true"></span>'
                        f'<code>{index:02d}</code><span>{escape(label)}</span></div>')
    template = Path(__file__).with_name("templates").joinpath("code-preview.html").read_text(encoding="utf-8")
    substitutions = {"styles": "\n".join(theme_styles + role_styles), "samples": render_samples(samples),
                     "metrics": "".join(metrics), "swatches": "".join(swatches)}
    return re.sub(r"\{\{(styles|samples|metrics|swatches)\}\}", lambda match: substitutions[match[1]], template)
