"""Contrast requirements for the shared design foundation."""

from dataclasses import dataclass

from .contract import ANSI, SURFACES, THEMES
from .code_roles import ROLES


@dataclass
class Report:
    tokens: int = 0
    contrast_checks: int = 0


def contrast_rules(color_names):
    """Required pairs from the token contract and usage notes; no rounded thresholds."""
    rules = {}

    def add(foreground, backgrounds, minimum, themes=THEMES):
        for background in backgrounds:
            for theme in themes:
                key = (foreground, background, theme)
                rules[key] = max(minimum, rules.get(key, 0))

    for name in ("ink", "ink-muted", "accent", "accent-hover", "frost", "fern"):
        add(name, SURFACES, 4.5)
    add("ink-subtle", ["bg-base", "bg-surface", "bg-raised"], 4.5)
    add("line-strong", ["bg-base", "bg-surface"], 3)
    add("focus-ring", SURFACES, 3)
    for foreground, background in (
        ("ink-inverse", "bg-inverse"), ("on-accent", "accent"),
        ("on-accent", "accent-hover"), ("accent-ink", "accent-soft"),
        ("on-danger", "danger"),
    ):
        add(foreground, [background], 4.5)
    for status in ("info", "success", "warning", "danger"):
        add(status, [status + "-soft"], 4.5)
        add("ink", [status + "-soft"], 4.5)
    add("ink", ["selection", "diff-add-bg", "diff-del-bg", "search-match", "search-match-current"], 4.5)
    for name in sorted(color_names):
        if name.startswith("syn-"):
            add(name, ["bg-base", "bg-sunken", "bg-surface"], 4.5)
    add("syn-comment", ["bg-base"], 5)
    add("syn-comment", ["bg-base"], 7.5, ("night",))
    # ANSI 0 is a background role in dark themes, not normal reading text.
    for name in ANSI[1:]:
        add(name, ["bg-base"], 4.5)
    add("ansi-black", ["bg-base"], 4.5, ("day",))
    for role in ROLES.values():
        add(role.color, role.backgrounds, role.minimum)
    return rules
