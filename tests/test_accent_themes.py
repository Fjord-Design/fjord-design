from pathlib import Path

import pytest

from fjord.contract import MODES
from fjord.project import Project


ROOT = Path(__file__).resolve().parents[1]
ACCENT_ROLES = ("accent", "accent-hover", "accent-soft", "accent-ink", "selection")
THEMES = {
    "fern": ("fern", "ansi-bright-green", "success-soft", "success", "diff-add-bg"),
    "water": ("frost", "ansi-bright-blue", "info-soft", "frost", "info-soft"),
    "mountain": ("lilac", "ansi-bright-magenta", "lilac-soft", "lilac", "lilac-soft"),
    "cabin": ("latte", "ansi-bright-yellow", "warning-soft", "latte", "warning-soft"),
    "hearth": ("hearth", "hearth-hover", "hearth-soft", "hearth", "hearth-soft"),
}


@pytest.mark.parametrize("mode", MODES)
@pytest.mark.parametrize("theme, targets", THEMES.items())
def test_selected_accents_preserve_every_other_resolved_value(mode, theme, targets):
    project = Project(ROOT)
    original = project.compose(mode)
    composed = project.compose(mode, ROOT / f"examples/themes/{theme}.tokens.json")
    roles = {role: original.values[f"semantic.color.{target}"]
             for role, target in zip(ACCENT_ROLES, targets)}
    expected = dict(original.values)
    expected.update({f"semantic.color.{role}": value for role, value in roles.items()})
    for path, role in {
        "semantic.color.accent-pressed": "accent-hover",
        "semantic.color.focus-ring": "accent",
        "component.button.primary.background": "accent",
        "component.button.primary.hover-background": "accent-hover",
        "component.button.primary.pressed-background": "accent-hover",
        "component.button.primary.focus-color": "accent",
    }.items():
        expected[path] = roles[role]
    assert composed.values == expected


