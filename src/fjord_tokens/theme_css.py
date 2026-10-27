"""Build scoped identity colours for the Svelte reference."""

import re

from .contract import THEMES
from .dtcg import TokenError
from .project import Project, check_contrast
from .semantics import declarations, variable


def theme_css(root, identity, theme=None, overrides=()):
    if not re.fullmatch(r"[a-z][a-z0-9]*(?:-[a-z0-9]+)*", identity or ""):
        raise TokenError("identity must use lower-case letters, digits and hyphens")
    project = Project(root)
    modes = {}
    for mode in THEMES:
        base = project.compose(mode)
        composition = project.compose(mode, theme, overrides)
        check_contrast(composition)
        changed = [path for path in composition.tokens
                   if composition.tokens[path]["$value"] != base.tokens[path]["$value"]
                   and not path.startswith("semantic.color.")]
        if changed:
            raise TokenError(f"scoped identity export supports semantic colour overrides only: {', '.join(sorted(changed))}")
        paths = [path for path in composition.values
                 if path.startswith(("semantic.color.", "semantic.shadow.", "component."))]
        if len({variable(path) for path in paths}) != len(paths):
            raise TokenError("scoped identity CSS contains duplicate names")
        values = {path: {"type": composition.tokens[path]["$type"], "value": composition.values[path]}
                  for path in sorted(paths)}
        modes[mode] = {"values": values}
    result = [f"/* Fjord scoped identity: {identity}; sRGB HEX. */"]
    for mode, values in modes.items():
        result.append(f'[data-fjord-mode="{mode}"][data-fjord-identity="{identity}"] '
                      + "{" + declarations(values, mode) + "}")
    selector = f'[data-fjord-preference="system"][data-fjord-mode][data-fjord-identity="{identity}"]'
    result.append("@media (prefers-color-scheme:light) {" + selector + " {" + declarations(modes["day"], "day") + "}}")
    return "\n".join(result) + "\n"
